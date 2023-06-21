use logos::{Logos, Source};
use std::{path::{Path,PathBuf}, rc::Rc, vec, fmt::Formatter, ops::{Range, RangeInclusive}, fs::read_to_string};
use super::MmgxError;

#[derive(Clone)]
pub struct LexerInfo {
    line: usize,
    line_start: usize,
    line_start_last: usize,
    offset: usize,
    path: PathBuf
}

impl LexerInfo {
    fn get(lex: &logos::Lexer<CodeToken>) -> Self
    {
        let mut obj = lex.extras.clone();
        obj.offset = lex.span().start;
        obj
    }
    fn line(lex: &logos::Lexer<CodeToken>) -> Option<String>
    {
        lex.source().slice(lex.extras.line_start..lex.span().end).map(|s| s.to_string())
    }
    fn line_from(&self, lex: &logos::Lexer<CodeToken>) -> Option<String>
    {
        lex.source().slice(self.line_start..lex.span().end).map(|s| s.to_string())
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(extras = LexerInfo)]
pub enum CodeToken{

    #[regex("[a-zA-Z][a-zA-Z0-9]*", priority = 2)]
    Name,

    #[regex("[a-zA-Z_][a-zA-Z_0-9]*")]
    Identifier,

    #[regex("\"([^\"\\\\]\\.)*\"")]
    String,

    #[regex("//.*")]
    Comment,

    #[regex("[0-9]+")]
    Number,

    #[token("<")]
    DiamondOpen,
    #[token(">")]
    DiamondClose,

    #[token("(")]
    ParenOpen,
    #[token(")")]
    ParenClose,

    #[token("{")]
    CurlyOpen,
    #[token("}")]
    CurleyClose,

    #[token("@")]
    Modifier,

    #[token("...")]
    VaArgs,

    #[token("..")]
    Range,

    #[token(".")]
    Dot,

    #[token("::")]
    ScopeResolution,

    #[token("\r\n", |lex| lex.extras.line+=1; lex.extras.line_start_last=lex.extras.line_start; lex.extras.line_start=lex.span().end)]
    #[token("\n", |lex| lex.extras.line+=1; lex.extras.line_start_last=lex.extras.line_start; lex.extras.line_start=lex.span().end)]
    NewLine,

    #[token(",")]
    Comma,

    #[regex(r"[ \t]+")]
    Whitespace,

    Unknown,
}

struct UnexpectedToken {
    got: Result<CodeToken, ()>,
    expected: Vec<CodeToken>,
}

struct WrongArgument {
    function: String,
    got: String,
    expected: Vec<String>
}

enum ParseErrorKind {
    UnexpectedToken(UnexpectedToken),
    UnexpectedEOF(Vec<CodeToken>),
    WrongArgument(WrongArgument),
    UnknownCommand(String)
}

pub struct ParseError {
    line: usize,
    line_start: usize,
    span: Range<usize>,
    line_str: Option<String>,
    kind: ParseErrorKind,
}

impl ParseError {
    fn create(lex: &logos::Lexer<CodeToken>, kind: ParseErrorKind) -> Self
    {
        Self { line: lex.extras.line,
               line_start: lex.extras.line_start,
               span: lex.span(),
               line_str: LexerInfo::line(lex),
               kind
        }
    }

    fn unexpected_token(lex: &logos::Lexer<CodeToken>, got: Option<Result<CodeToken, ()>>, expected: Vec<CodeToken>) -> Self
    {
        let mut line = lex.extras.line;
        let mut line_start= lex.extras.line_start;

        let reason = match got {
            Some(t) => {
                if t==Ok(CodeToken::NewLine) {
                    line-=1;
                    line_start = lex.extras.line_start_last;
                };
                ParseErrorKind::UnexpectedToken(UnexpectedToken { got: t, expected })
            },
            None => ParseErrorKind::UnexpectedEOF(expected)
        };

        Self { line,
               line_start,
               span: lex.span(),
               line_str:
                   if let Some(string) = lex.source().slice(line_start..lex.span().end) {
                       Some(String::from(string))
                   }else{
                       None
                   },
               kind: reason
        }
    }

    fn wrong_argument(lex: &logos::Lexer<CodeToken>, pos: LexerInfo, function: String, got: String, expected: Vec<String>) -> Self
    {
        Self { line: pos.line,
               line_start: pos.line_start,
               span: pos.offset..lex.span().end,
               line_str: pos.line_from(lex),
               kind: ParseErrorKind::WrongArgument(WrongArgument{function, got, expected})
        }
    }

    fn unknown_command(lex: &logos::Lexer<CodeToken>) -> Self
    {
        Self::create(lex, ParseErrorKind::UnknownCommand(String::from(lex.slice())))
    }

    pub fn fmt(&self, f: &mut Formatter, path: &Path) -> std::fmt::Result {
        write!(f, "error at {}:{}:{} ", path.to_str().unwrap(), self.line, self.span.start-self.line_start)?;
        match &self.kind {
            ParseErrorKind::UnexpectedToken(ut) =>{
                write!(f, "Unexpected Token got {:?}", ut.got.as_ref().unwrap_or(&CodeToken::Unknown))?;
                if let Some(line) = &self.line_str {
                    write!(f, "({:?})", line.slice((self.span.start-self.line_start)..(self.span.end-self.line_start)).unwrap())?;
                }
                writeln!(f, " expected {:?}", ut.expected)?;
            },
            ParseErrorKind::UnexpectedEOF(expected) => {
                writeln!(f, "Reached End of File expected Token {:?}", expected)?;
            },
            ParseErrorKind::WrongArgument(wa) => {
                writeln!(f, "Wrong Argument for {} got {} expected {:?}", wa.function, wa.got, wa.expected)?;
            },
            ParseErrorKind::UnknownCommand(cmd) => {
                writeln!(f, "Unknown Command '{}'", cmd)?;
            }

        };

        // Mark error on faulty line
        if let Some(line) = &self.line_str {
            f.write_str(line.as_str())?;

            let mut tmp = String::from("\n");
            for i in 0..(self.span.start-self.line_start) {
                match line.slice(i..(i+1)).unwrap() {
                    "\t" => tmp+="     ",
                    "\n" | "\r" => tmp.clear(),
                    _ => tmp+=" "
                }
            };
            for _ in self.span.clone() {
                tmp+="~";
            };
            f.write_str(tmp.as_str())?;
            f.write_str("\n")?;
        } else {
            writeln!(f, "[No line information]")?;
        }

        Ok(())
    }
}



#[derive(Debug, Clone)]
pub struct External{
    pub module: String,
    pub object: String,
    pub implement: bool
}

#[derive(Debug)]
pub struct TemplateCall {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub enum BodyStatement {
    Expand(String),
    Parameter(usize),
    OptDependency(String),
    External(External),
    TemplateCall(TemplateCall),
}


#[derive(Debug)]
pub struct Object{
    pub name: String,
    pub body: Vec<BodyStatement>
}

#[derive(Debug, PartialEq)]
pub enum TemplateParameter {
    Param(String),
    Reference(usize)
}

#[derive(Debug)]
pub struct Template {
    pub params: Vec<TemplateParameter>,
    pub obj: Object
}

#[derive(Debug)]
pub struct CommandImpl {
    pub template: String,
    pub params: Vec<RangeInclusive<usize>>
}

#[derive(Debug)]
pub enum Command {
    Impl(CommandImpl),
    Use(String),
    Export(Object)
}

#[derive(Debug)]
pub enum Statement {
    Object(Object),
    Template(Template),
    Command(Command),
    Comment(String)
}

pub struct MmgxModule {
    pub name: String,
    pub body: Vec<Statement>,
    pub line: usize
}

pub enum Section {
    CSource(String),
    MmgxModule(Rc<MmgxModule>),
    // MmgxCall
}

fn lex_next(lex: &mut logos::Lexer<CodeToken>) -> Option<Result<CodeToken, ()>>
{
    let token = lex.next();
    // println!("[token: {:?} -> {:?}]", token, lex.slice());
    token
}
fn next_non_whitespace(lex: &mut logos::Lexer<CodeToken>) -> Option<Result<CodeToken,()>>
{
    loop{
        match lex_next(lex) {
            Some(Ok(CodeToken::Whitespace)) |
            Some(Ok(CodeToken::Comment)) =>{},
            t @ _ =>{return t;}
        };
    }
}

fn parse_mmgx_parameters_body<R, const N: usize, const V: usize, F: Fn([&str; N]) -> Result<R, ParseError> >(lex: & logos::Lexer<CodeToken>, tokens: Vec<(Result<CodeToken, ()>, &str)>, capture: &[[CodeToken; N]; V], func: &F) -> Result<R, ParseError>
{

    let mut found = false;

    let mut tmp = [""; N];

    let mut expect = Vec::new();
    let mut token = None;

    for c in 0..V {

        let mut iter = tokens.iter();
        found = true;

        for i in 0..N {
            if let Some(t) = iter.next() {
                token = Some(t.0.clone());
                if t.0.clone().map_or(false, |x| x == capture[c][i]) {
                    tmp[i] = t.1;
                } else {
                    expect.push(capture[c][i].clone());
                    found = false;
                    break;
                }
            }
        }
        if found {break};
    };

    if !found {
        for c in capture {
            if !c.is_empty(){
                expect.push(c[0].clone());
            }
        }
        return Err(ParseError::unexpected_token(lex, token, expect));
    }

    func(tmp)
}

fn parse_mmgx_parameters<R, const N: usize, const V: usize, F: Fn([&str; N]) -> Result<R, ParseError> >(lex: &mut logos::Lexer<CodeToken>, capture: [[CodeToken; N]; V], func: F, end: CodeToken) -> Result<Vec<R>, ParseError>
{
    let mut params = Vec::new();

    let mut tokens: Vec<(Result<CodeToken, ()>, &str)> = Vec::new();

    while let Some(t) = next_non_whitespace(lex) {
        match t {
            Ok(CodeToken::Comma) => {
                params.push(parse_mmgx_parameters_body(lex, tokens, &capture, &func)?);
                tokens = Vec::new();
            },
            _ => {
                if t.as_ref().ok().map_or(false, |x| x==&end) {
                    params.push(parse_mmgx_parameters_body(lex, tokens, &capture, &func)?);
                    break;
                } else {
                    tokens.push((t, lex.slice()));
                }
            }
        }
    };
    Ok(params)
}

fn parse_mmgx_object_body<'a>(tokens: &Vec<(Option<Result<CodeToken, ()>>, &'a str)>, i: &mut usize, params: &Option<Vec<TemplateParameter>>, args: &Option<Vec<String>>) -> Option<BodyStatement>
{
    let t1 = tokens.get(*i).unwrap_or(&(None, ""));
    let t2 = tokens.get(*i+1).unwrap_or(&(None, ""));
    let t3 = tokens.get(*i+2).unwrap_or(&(None, ""));

    let mut implement = None;

    // [MODULE]::[OBJECT]
    // [MODULE].[OBJECT]

    match t2.0 {
        Some(Ok(CodeToken::Dot)) => implement = Some(true),
        Some(Ok(CodeToken::ScopeResolution)) => implement = Some(false),
        Some(Ok(CodeToken::DiamondOpen)) => {
            let mut arg_list: Vec<&String> = Vec::new();

            if let Some(params) = params {
                for p in params {
                    match p {
                        TemplateParameter::Param(string) => arg_list.push(string),
                        _=>{}
                    }
                }
            }
            if let Some(args) = args {
                for s in args {
                    arg_list.push(s);
                }

            }

            let mut list = Vec::new();
            let mut index = 0;

            let tokens: Vec<&(Option<Result<CodeToken, ()>>, &str)> = tokens.iter().skip(*i+2).filter(|e| !matches!(e.0, Some(Ok(CodeToken::Whitespace)))).collect();

            while index < tokens.len()-1 {
                match tokens[index].0 {
                    Some(Ok(CodeToken::Name)) => {
                        if let Some(e) = arg_list.iter().find(|e| **e==&String::from(tokens[index].1)) {
                            list.push((*e).clone());
                        } else {
                            return None;
                        }
                    },
                    _ => {return None;}
                }
                match tokens[index+1].0 {
                    Some(Ok(CodeToken::Comma)) => {},
                    Some(Ok(CodeToken::DiamondClose)) => break,
                    _ => {return None;}
                };
                index+=2;
            }
            match t1.0 {
                Some(Ok(CodeToken::Name)) => {
                    *i+=index+3;
                    return Some(BodyStatement::TemplateCall(TemplateCall { name: String::from(t1.1), args: list }));
                },
                _ => {return None;}
            }
        },
        _ => {return None;}
    };

    if let Some(implement) = implement {
        match t3.0 {
        Some(Ok(CodeToken::Name)) => {
            let res = Some(BodyStatement::External(
                External {
                    module: match t1.0 {
                        Some(Ok(CodeToken::Name)) | Some(Ok(CodeToken::Identifier)) => String::from(t1.1),
                        _ => {return None}
                        },
                    object: String::from(t3.1),
                    implement
                    }
                ));

                *i+=2;
                return res;
            },
            _ => {}
        }
    };

    None
}

fn parse_body(tokens: Vec<(Option<Result<CodeToken, ()>>, &str)>, params: &Option<Vec<TemplateParameter>>, args: &Option<Vec<String>>, va_args: bool) -> Vec<BodyStatement>
{

    let mut tmp = String::new();
    let mut res = Vec::new();

    let mut i = 0;

    while i<tokens.len() {
        let t = &tokens[i];
        match t.0 {
            Some(Ok(CodeToken::VaArgs)) => tmp.push_str(if va_args {"__VA_ARGS__"} else {t.1}),
            Some(Ok(CodeToken::Name))=> {

                // flush buffer
                res.push(BodyStatement::Expand(tmp));
                tmp = String::new();

                if let Some(vec) = &params {
                    if let Some(index) = vec.iter().position(|e| *e == TemplateParameter::Param(String::from(t.1))) {
                        res.push(BodyStatement::Parameter(index));
                        i+=1;
                        continue;
                    }
                }

                match parse_mmgx_object_body(&tokens, &mut i, params, args) {
                    Some(e) => {
                        res.push(e)
                    },
                    None => res.push(BodyStatement::OptDependency(String::from(t.1)))
                }
            },
            _ => tmp.push_str(t.1)
        }
        i+=1;
    };
    res.push(BodyStatement::Expand(tmp));
    res
}

fn parse_mmgx_object(lex: &mut logos::Lexer<CodeToken>) -> Result<Statement, ParseError>
{
    let name = String::from(lex.slice());
    let mut params = None;
    let mut args = None;

    let mut body = Vec::new();

    let mut va_args = false;

    let mut body_tokens = Vec::new();

    let mut skip_body = false;

    match lex_next(lex) {
        Some(Ok(CodeToken::DiamondOpen)) => {
            let list = parse_mmgx_parameters(lex, [[CodeToken::Name]], |name| Ok(String::from(name[0])), CodeToken::DiamondClose)?;

            let mut res = Vec::new();

            for p in list {
                if let Some(index) = res.iter().position(|e| match e{TemplateParameter::Param(name) => name==&p, _=> false}) {
                    res.push(TemplateParameter::Reference(index));
                } else {
                    res.push(TemplateParameter::Param(p));
                }
            }
            params = Some(res);
        },
        Some(Ok(CodeToken::ParenOpen)) => {
            let list = parse_mmgx_parameters(lex, [[CodeToken::Name], [CodeToken::VaArgs]], |name| Ok(String::from(name[0])), CodeToken::ParenClose)?;

            va_args = list.contains(&String::from("..."));

            // feed back args
            body.push(BodyStatement::Expand(String::from("(")));
            body.push(BodyStatement::Expand(list.join(", ") + ")"));

            args = Some(list);
        },
        Some(Ok(CodeToken::NewLine)) => skip_body=true,
        t @ _ => body_tokens.push((t, lex.slice()))
    };
    match lex_next(lex) {
        Some(Ok(CodeToken::ParenOpen)) => {
            let args = parse_mmgx_parameters(lex, [[CodeToken::Name], [CodeToken::VaArgs]], |name| Ok(String::from(name[0])), CodeToken::ParenClose)?;

            va_args = args.contains(&String::from("..."));

            // feed back args
            body.push(BodyStatement::Expand(String::from("(")));
            body.push(BodyStatement::Expand(args.join(", ") + ")"));
        },
        Some(Ok(CodeToken::NewLine)) => skip_body=true,
        t @ _ => body_tokens.push((t, lex.slice()))
    };

    if !skip_body {
        while let Some(t) = lex_next(lex) {
            match t {
                Ok(CodeToken::NewLine) => break,
                _ => body_tokens.push((Some(t), lex.slice()))
            };
        };
    }
    body.append(&mut parse_body(body_tokens, &params, &args, va_args));
    let obj = Object {name, body};
    match params {
        Some(params) => Ok(Statement::Template(Template { params, obj })),
        None => Ok(Statement::Object(obj))
    }
}

fn parse_mmgx_impl(lex: &mut logos::Lexer<CodeToken>) -> Result<CommandImpl, ParseError>
{
    match next_non_whitespace(lex) {
        Some(Ok(CodeToken::Name)) => {

            let template = lex.slice();
            let token = lex_next(lex);
            if token == Some(Ok(CodeToken::DiamondOpen)) {
                let params = parse_mmgx_parameters(lex, [[CodeToken::Number, CodeToken::Range, CodeToken::Number]], |tokens| Ok( tokens[0].parse::<usize>().unwrap() ..= tokens[2].parse().unwrap() ), CodeToken::DiamondClose)?;

                match lex_next(lex) {
                    Some(Ok(CodeToken::NewLine)) => Ok(CommandImpl{template: String::from(template), params}),
                    t @ _ => Err(ParseError::unexpected_token(lex, t, vec![CodeToken::Name]))
                }
            } else {
                Err(ParseError::unexpected_token(lex, token, vec![CodeToken::DiamondOpen]))
            }
        },
        t @ _ => Err(ParseError::unexpected_token(lex, t, vec![CodeToken::Name]))
    }

}
fn parse_mmgx_command(lex: &mut logos::Lexer<CodeToken>) -> Result<Statement, ParseError>
{
    match lex_next(lex) {
        Some(Ok(CodeToken::Name)) => {

            match lex.slice() {
                "export" => {
                    // Get Object
                    next_non_whitespace(lex);
                    let position = LexerInfo::get(lex);

                    match parse_mmgx_object(lex)? {
                        Statement::Object(obj) => Ok(Statement::Command(Command::Export(obj))),
                        t @ _ => Err(ParseError::wrong_argument(lex, position, String::from("@export"), format!("{:?}", t), vec![String::from("Object")]))
                    }
                },
                "use" => {
                    match next_non_whitespace(lex) {
                        Some(Ok(CodeToken::Identifier)) | Some(Ok(CodeToken::Name)) => Ok(Statement::Command(Command::Use(String::from(lex.slice())))),
                        t @ _ => Err(ParseError::unexpected_token(lex, t, vec![CodeToken::Identifier, CodeToken::Name]))
                    }

                }
                "impl" => Ok(Statement::Command(Command::Impl(parse_mmgx_impl(lex)?))),
                _ => Err(ParseError::unknown_command(lex))
            }
        },
        t @ _ => Err(ParseError::unexpected_token(lex, t, vec![CodeToken::Name]))
    }
}

fn parse_mmgx_body(lex: &mut logos::Lexer<CodeToken>) -> Result<Vec<Statement>, ParseError>
{
    let mut statements = Vec::new();
    loop {
        match lex_next(lex) {
            Some(Ok(CodeToken::Name)) => statements.push(parse_mmgx_object(lex)?),
            Some(Ok(CodeToken::Modifier)) => {
                statements.push(parse_mmgx_command(lex)?);
            },
            Some(Ok(CodeToken::CurleyClose)) => {return Ok(statements);},
            Some(Ok(CodeToken::Whitespace)) |
            Some(Ok(CodeToken::NewLine)) => continue,
            Some(Ok(CodeToken::Comment)) => statements.push(Statement::Comment(String::from(lex.slice()))),
            t @ _ => {return Err(ParseError::unexpected_token(lex, t, vec![CodeToken::Name, CodeToken::Modifier, CodeToken::CurleyClose]));}
        }
    }
}

fn parse_mmgx_module(lex: &mut logos::Lexer<CodeToken>) -> Result<MmgxModule, ParseError>
{
    match next_non_whitespace(lex) {
        Some(Ok(CodeToken::Identifier)) | Some(Ok(CodeToken::Name)) =>{
            let name = lex.slice();
            let token = next_non_whitespace(lex);
            if token == Some(Ok(CodeToken::CurlyOpen)) {

                Ok(MmgxModule{name: String::from(name), body: parse_mmgx_body(lex)?, line: lex.extras.line})

            }else{
                Err(ParseError::unexpected_token(lex, token, vec![CodeToken::CurlyOpen]))
            }
        },
        t @ _ => Err(ParseError::unexpected_token(lex, t, vec![CodeToken::Identifier]))
    }
}

pub fn parse_x_file(path: &PathBuf) -> Result<Vec<Section>, MmgxError>
{
    let source = match  read_to_string(path) {
        Ok(s) => s,
        Err(e) =>{ return Err(MmgxError::file_read_error(path, e));}
    };

    let mut lex = CodeToken::lexer_with_extras(source.as_str(), LexerInfo {line: 0, line_start: 0, line_start_last: 0, path: path.into(), offset: 0});
    let mut file = Vec::new();

    while let Some(token) = lex_next(&mut lex) {
        match token {
            Ok(CodeToken::Modifier) =>{
                file.push(Section::MmgxModule(Rc::new(parse_mmgx_module(&mut lex)
                        .map_err(|error| MmgxError::parse_error(&lex.extras.path, error))?)))
            },
            _ =>{
                match file.last_mut() {
                    Some(Section::CSource(string)) => string.push_str(lex.slice()),
                    _ => file.push(Section::CSource(String::from(lex.slice())))
                };
            }
        }
    };
    Ok(file)
}
