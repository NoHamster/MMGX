use std::{path::{Path,PathBuf}, vec};
use clap::Parser;
mod compile;

#[derive(Parser)]
struct Args {
    files: Vec<PathBuf>,

    #[arg(long, short, default_value_t = String::from("c"))]
    extension: String
}


fn main() {

    let args = Args::parse();

    match compile::files(args.files, args.extension){
        Ok(()) => (),
        Err(me) => println!("An error occured: {}", me)
    };
}
