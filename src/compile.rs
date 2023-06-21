mod parse;
use std::{path::{Path, PathBuf}, fmt::{Display, Formatter}, fs::File, io::prelude::*, ops::{Range, RangeInclusive}, rc::Rc};

enum CompileErrorKind {
    NoObjectInModule(parse::External),
    ModuleNotIncluded(String),
    NoTemplateParameters,
    TemplateParameterOutOfRange((usize, usize)),
    ErrorWhileCompiling((String, Box<CompileErrorKind>)),
    ModuleNotFound(String),
    TemplateNotFound(String)
}

pub struct CompileError {
    line: usize,
    error: CompileErrorKind
}

impl CompileError {

    fn fmt_err(f: &mut std::fmt::Formatter, error: &CompileErrorKind, indent: usize) -> std::fmt::Result
    {
        for _ in 0..indent {
            write!(f, "\t")?;
        }
        match error {
            CompileErrorKind::NoObjectInModule(ext) => writeln!(f, "Object {} not found in Module {}", ext.object, ext.module),
            CompileErrorKind::ModuleNotIncluded(srting) => writeln!(f, "Module {} has not been included! Include modules with @use [Module]", srting),
            CompileErrorKind::NoTemplateParameters => writeln!(f, "Object has no Template Parameters!"),
            CompileErrorKind::ErrorWhileCompiling((string, error)) => {
                writeln!(f, "While Compiling {}:", string)?;
                Self::fmt_err(f, error.as_ref(), indent+1)
            },
            CompileErrorKind::TemplateParameterOutOfRange((got, max)) => writeln!(f, "Tried indexing template Parameter {} but only {} are specified!", got, max),
            CompileErrorKind::TemplateNotFound(name) => writeln!(f, "Template not found {}", name),
            CompileErrorKind::ModuleNotFound(name) => writeln!(f, "Module not found {}", name),
        }
    }
    fn fmt(&self, f: &mut std::fmt::Formatter, path: &Path) -> std::fmt::Result {
        write!(f, "error at {}:{} ", path.to_str().unwrap(), self.line)?;
        Self::fmt_err(f, &self.error, 0)
    }
}

enum MmgxErrorKind {
    ParseError(parse::ParseError),
    FileReadError(String),
    FileOpenError(String),
    FileWriteError(String),
    CompileError(CompileError)
}

pub struct MmgxError {
    path: PathBuf,
    error: MmgxErrorKind
}

impl Display for MmgxError {

    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result
    {
        match &self.error {
            MmgxErrorKind::ParseError(e) =>{
                writeln!(f, "Parsing failed!")?;
                e.fmt(f, &self.path)
            },
            MmgxErrorKind::FileReadError(reason) => write!(f, "Could not read file '{}': {}", self.path.to_str().unwrap(), reason),
            MmgxErrorKind::FileOpenError(reason) => write!(f, "Could not open file '{}': {}", self.path.to_str().unwrap(), reason),
            MmgxErrorKind::FileWriteError(reason) => write!(f, "Could not write file '{}': {}", self.path.to_str().unwrap(), reason),
            MmgxErrorKind::CompileError(cmp) => {
                writeln!(f, "Compiling failed!")?;
                cmp.fmt(f, &self.path)
            }
        }
    }

}

impl MmgxError {

    pub fn file_read_error(path: &PathBuf, error: std::io::Error) -> Self
    {
        Self{path: path.into(), error: MmgxErrorKind::FileReadError(error.to_string())}
    }

    pub fn file_open_error(path: &PathBuf, error: std::io::Error) -> Self
    {
        Self{path: path.into(), error: MmgxErrorKind::FileOpenError(error.to_string())}
    }

    pub fn file_write_error(path: &PathBuf, error: std::io::Error) -> Self
    {
        Self{path: path.into(), error: MmgxErrorKind::FileWriteError(error.to_string())}
    }

    pub fn parse_error(path: &PathBuf, error: parse::ParseError) -> Self
    {
        Self{path: path.into(), error: MmgxErrorKind::ParseError(error)}
    }

    pub fn compile_error(path: &PathBuf, error: CompileError) -> Self
    {
        Self {path: path.into(), error: MmgxErrorKind::CompileError(error)}
    }
}

trait Compile{
    type Args<'a>;

    fn internal_compile<'a>(&self, args: Self::Args<'a>) -> Result<String, CompileErrorKind>;
    fn compile<'a>(&self, args: Self::Args<'a>) -> Result<String, CompileErrorKind>
    {
        self.internal_compile(args).map_err(|err| CompileErrorKind::ErrorWhileCompiling((self.name().clone(), Box::new(err))))
    }
    fn name(&self) -> &String;
}

impl parse::Object {
    fn get_name(&self, prefix_name: &String, prefix: bool) -> String
    {
        format!("{}{}_{}", if prefix{"__"}else{""}, prefix_name, self.name)
    }
    fn resolve(name: &String, parent: &parse::MmgxModule, prefix_name: &String) -> String
    {
        for e in parent.body.iter() {
            match e {
                parse::Statement::Template(temp) => {
                    if &temp.obj.name==name {
                        return temp.obj.get_name(prefix_name, true);
                    }
                },
                parse::Statement::Object(obj) => {
                    if &obj.name==name {
                        return obj.get_name(prefix_name, true);
                    }
                },
                parse::Statement::Command(cmd) => match cmd {
                    parse::Command::Export(obj) => {
                        if &obj.name==name {
                            return obj.get_name(prefix_name, false);
                        }
                    },
                    _=>{}
                },
                _ => {}
            }
        }
        name.clone()
    }
}

impl Compile for parse::Object
{
    type Args<'a> = (&'a parse::MmgxModule, &'a String, bool, &'a Vec<&'a Rc<parse::MmgxModule>>, Option<(&'a Vec<usize>, &'a Vec<parse::TemplateParameter>)>);
    fn name(&self) -> &String
    {
        &self.name
    }

    fn internal_compile<'a>(&self, (parent, prefix_name, prefix, external, params): Self::Args<'a>) -> Result<String, CompileErrorKind>
    {
        let mut res = format!("#define {}", self.get_name(prefix_name, prefix));

        if let Some((val, map)) = params {
            let mut i=0;
            for p in map {
                res+="_";
                match p {
                    parse::TemplateParameter::Param(_) => {res+=val[i].to_string().as_str(); i+=1},
                    parse::TemplateParameter::Reference(i) => res+=val[*i].to_string().as_str()
                }
            }
        }

        for e in self.body.iter() {
            match e {
                parse::BodyStatement::Expand(string) => res+=string.as_str(),
                parse::BodyStatement::External(ext) => {
                    if let Some(module) = external.iter().find(|e| e.name==ext.module) {

                        if let Some(parse::Statement::Object(obj)) = module.body.iter()
                            .find(|e| match e {
                                parse::Statement::Object(obj) => obj.name==ext.object,
                                parse::Statement::Command(cmd) => match cmd {
                                    parse::Command::Export(obj) => obj.name==ext.object,
                                    _=> false
                                }
                                _ => false
                        }) {
                            if ext.implement {
                                let prefix = format!("__{}_{}_IMPL_{}", prefix_name, self.name, ext.module);
                                res = obj.compile((parent, &prefix, false, external, None))? + res.as_str() + prefix.as_str();
                            } else {
                                res+= format!("__{}_{}", ext.module, ext.object).as_str();
                            }

                        } else {
                            return Err(CompileErrorKind::NoObjectInModule(ext.clone()));
                        }
                    } else {
                        return Err(CompileErrorKind::ModuleNotIncluded(ext.module.clone()));

                    }
                },
                parse::BodyStatement::Parameter(idx) => {
                    if let Some(parameters) = &params {

                        if let Some(num) = parameters.0.get(*idx) {
                            res+=num.to_string().as_str();
                        } else {
                            return Err(CompileErrorKind::TemplateParameterOutOfRange((*idx, parameters.0.len())));
                        }
                    } else {
                        return Err(CompileErrorKind::NoTemplateParameters);
                    }
                },
                parse::BodyStatement::OptDependency(string) => {
                    res += Self::resolve(string, parent, prefix_name).as_str();
                },
                parse::BodyStatement::TemplateCall(call) => {
                    res+= Self::resolve(&call.name, parent, prefix_name).as_str();
                    for v in &call.args {
                        let mut found = false;
                        if let Some(params) = params {
                            for p in params.1.iter().enumerate() {
                                if match p.1 {
                                    parse::TemplateParameter::Param(string) => string==v,
                                    _=> false
                                } {
                                    res+="_";
                                    res+=params.0[p.0].to_string().as_str();
                                    found=true;
                                }
                            }
                        }
                        if !found {
                            res+="##_##";
                            res+=v.as_str();
                        }
                    }
                }
            }
        };
        Ok(res+"\n")
    }
}

impl parse::MmgxModule {

    fn recursive_impl(res: &mut String, template: &parse::Template, parent: &parse::MmgxModule, prefix_name: &String, prefix: bool, external: &Vec<&Rc<parse::MmgxModule>>, ranges: &Vec<RangeInclusive<usize>>, param: &mut Vec<usize>, idx: usize) -> Result<(), CompileErrorKind>
    {
        if idx < ranges.len() {
            for i in ranges.get(idx).unwrap().clone() {
                param[idx] = i;
                Self::recursive_impl(res, template, parent, prefix_name, prefix, external, ranges, param, idx+1)?;
            }
        } else {
            res.push_str(template.obj.compile((parent, prefix_name, prefix, external, Some((param, &template.params))))?.as_str());
        };
        Ok(())
    }
}

impl Compile for parse::MmgxModule {
    type Args<'a> = &'a Vec<Rc<parse::MmgxModule>>;

    fn name(&self) -> &String
    {
        &self.name
    }


    fn internal_compile<'a>(&self, modules: Self::Args<'a>) -> Result<String, CompileErrorKind>
    {
        let mut res = String::new();
        let mut externs = Vec::new();
        let mut templates: Vec<&parse::Template> = Vec::new();

        for statement in self.body.iter() {
            match statement {
                parse::Statement::Command(cmd) => {
                    match cmd {
                        parse::Command::Use(name) => {
                            if let Some(module) = modules.iter().find(|e| &e.name == name) {
                                externs.push(module);
                            } else {
                                return Err(CompileErrorKind::ModuleNotFound(name.clone()));
                            }
                        },
                        parse::Command::Impl(cmd_impl) => {
                            if let Some(template) = templates.iter().find(|e| e.obj.name == cmd_impl.template) {

                                let imp_len = cmd_impl.params.len();
                                let tem_len = template.params.iter().filter(|e| matches!(e, parse::TemplateParameter::Param(_))).count();

                                if imp_len == tem_len {
                                    let mut args = Vec::new();
                                    args.resize(imp_len, 0);
                                    Self::recursive_impl(&mut res, template, &self, &self.name, true, &externs, &cmd_impl.params, &mut args, 0)?;
                                } else {
                                    return Err(CompileErrorKind::TemplateParameterOutOfRange((imp_len, tem_len)));
                                }
                            } else {
                                return Err(CompileErrorKind::TemplateNotFound(cmd_impl.template.clone()))
                            }
                        },
                        parse::Command::Export(obj) => {
                            res.push_str(obj.compile((&self, &self.name, false, &externs, None))?.as_str())
                        }
                    }
                },
                parse::Statement::Object(obj) => {
                    res.push_str(obj.compile((&self, &self.name, true, &externs, None))?.as_str());
                },
                parse::Statement::Template(temp) => {
                    templates.push(temp);
                },
                parse::Statement::Comment(string) => res+=string.as_str(),
            }
            res+="\n";
        };
        Ok(res)
    }
}

fn compile(files: Vec<(File, &PathBuf, Vec<parse::Section>)>) -> Result<(), MmgxError>
{
    let mut modules: Vec<Rc<parse::MmgxModule>> = Vec::new();

    // collect all Modules
    for file in files.iter() {
            modules.append(&mut file.2.iter()
                .filter_map(|sec| match sec {
                    parse::Section::MmgxModule(module) => Some(module.clone()),
                    _ => None
                })
                .collect());
    };

    for mut file in files {
        for section in file.2.iter() {
            let res = match section {
                    parse::Section::CSource(string) => string.clone(),
                    parse::Section::MmgxModule(module) => {
                        module.compile(&modules)
                        .map_err(|ek| MmgxError::compile_error(file.1, CompileError { line: module.line, error: ek}))?

                    }
                };
            file.0.write_all(res.as_bytes()).map_err(|err| MmgxError::file_write_error(file.1, err))?;
        }
    };
    Ok(())
}

pub fn files(input: Vec<PathBuf>, output: String) -> Result<(), MmgxError>
{

    let mut files = Vec::new();

    for path in &input {
        files.push((File::create(path.with_extension(output.as_str())).map_err(|err| MmgxError::file_open_error(path,err))?, path, parse::parse_x_file(path)?));
        // println!("File {} -> {:?}\n", path.to_str().unwrap(), files.last().unwrap().2);
    };

    compile(files)
}

