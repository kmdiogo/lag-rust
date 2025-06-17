mod lexer;
mod parse_tree_node;
mod parser;

use crate::lexer::Lexer;
use clap::Parser;
use std::fs;
use std::io::Read;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: String,
}

fn get_input(input_file: &str) -> (String, String) {
    (
        fs::read_to_string(input_file).expect("Unable to open file."),
        input_file.to_owned(),
    )
}

pub fn main() {
    env_logger::init();
    let args = Args::parse();
    let (text, filename) = get_input(args.input_file.as_ref());
    let lexer = Lexer::from_string(&text);
    let mut parser = parser::Parser::new(lexer);

    println!("Parsing...");
    match parser.parse() {
        Ok(false) => println!("Parsing failed."),
        Err(e) => {
            println!("Parsing error. See below for details: \n\t{}", e.message);
            println!("\t{}:{}:{}", &filename, e.token.line, e.token.col)
        }
        _ => {}
    }
}
