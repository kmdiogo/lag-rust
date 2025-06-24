mod arena;
mod dfa_serializer;
mod driver_generatable;
mod lexer;
mod parser;
mod regex_ast;

use crate::arena::ObjRef;
use crate::dfa_serializer::serialize_dfa;
use crate::driver_generatable::{DriverGeneratable, PythonDriverGenerator};
use crate::lexer::Lexer;
use crate::parser::parse;
use crate::regex_ast::{get_dfa, get_follow_pos, NodeRef, AST};
use clap::{Parser, ValueEnum};
use log::debug;
use std::fs::File;
use std::io::Write;
use std::{fmt, fs};

#[derive(ValueEnum, Clone, Debug)]
enum DriverLanguage {
    Python,
}

impl fmt::Display for DriverLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self) // or use a custom string if needed
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: String,

    #[arg(short, long, default_value = "python")]
    driver_language: DriverLanguage,
}

static python_driver_template: &'static str = include_str!("../driver_templates/python.py");

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
    let mut lexer = Lexer::from_string(&text);

    println!("Parsing...");
    let parse_output = match parse(&mut lexer) {
        Ok(output) => {
            println!("Parsing successful.");
            output
        }
        Err(e) => {
            println!("Parsing error. See below for details: \n\t{}", e.message);
            println!("\t{}:{}:{}", &filename, e.token.line, e.token.col);
            return;
        }
    };
    let ast = &parse_output.tree;
    if ast.size() == 0 {
        println!("No definitions found.");
        return;
    }
    debug!("Parse tree size {:?}", ast.get_pool().len());
    debug!("Node Ref Mapping:");
    debug!("End nodes: {:?}", &parse_output.end_nodes);
    for (node_ref, node) in ast.get_pool().iter().enumerate() {
        debug!(" {:?} => {:?}", ObjRef(node_ref as u32), node);
    }
    let root = ObjRef((ast.size() - 1) as u32);
    let meta = AST::get_meta(ast, root);
    let followpos = get_follow_pos(ast, &meta, root);
    debug!("Follow pos:");
    for (node_ref, node) in followpos.iter() {
        debug!("  {:?} => {:?}", node_ref, node);
    }
    let dfa_table = get_dfa(ast, &meta, &followpos, root);
    let mut file = File::create("states.json").unwrap();
    let json_string = serialize_dfa(
        &dfa_table,
        &meta.get(root as NodeRef).first_pos,
        &parse_output.end_nodes,
        &parse_output.token_order,
    );
    match file.write_all(json_string.as_bytes()) {
        Ok(_) => {}
        Err(why) => panic!("Error writing serialized DFA to JSON file: {}", why),
    };

    let user_defined_token_ids: Vec<String> = parse_output
        .token_order
        .into_iter()
        .filter(|token| token != "!")
        .collect();
    match args.driver_language {
        DriverLanguage::Python => {
            let mut driver_file = File::create("driver.py").unwrap();
            let contents = python_driver_template
                .replace(
                    "'__TOKEN_ENTRIES__'",
                    &PythonDriverGenerator::get_token_entries(&user_defined_token_ids),
                )
                .replace(
                    "'__STATE_TOKEN_MAPPING__'",
                    &PythonDriverGenerator::get_state_token_mapping(&user_defined_token_ids),
                );
            driver_file.write_all(contents.as_bytes()).unwrap();
        }
    }
}
