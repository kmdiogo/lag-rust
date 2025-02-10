mod lexer;
mod parser;

use crate::lexer::Lexer;
use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the input file
    #[arg(short, long)]
    input_file: String,
}

pub fn main() {
    env_logger::init();
    let args = Args::parse();

    let text_input = fs::read_to_string(&args.input_file).expect("Unable to open file.");
    let mut parser = parser::Parser::new(Lexer::from_string(&text_input));
    match parser.parse() {
        Ok(false) => println!("parsing failed."),
        Err(e) => {
            println!("error: {}", e.message);
            println!("\t{}:{}:{}", &args.input_file, e.token.line, e.token.col)
        }
        _ => {}
    }
}
