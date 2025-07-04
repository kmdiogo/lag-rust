mod arena;
mod dfa_serializer;
mod driver_generator;
mod lexer;
mod parser;
mod regex_ast;

use crate::arena::ObjRef;
use crate::dfa_serializer::serialize_dfa;
use crate::driver_generator::{
    generate_driver_content, DriverGeneratable, JavascriptDriverGenerator, PythonDriverGenerator,
};
use crate::lexer::Lexer;
use crate::parser::parse;
use crate::regex_ast::{get_dfa, get_follow_pos, ASTMeta, NodeRef, AST, DFA};
use clap::{Parser, ValueEnum};
use log::debug;
use std::collections::{BTreeSet, HashMap};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::{fmt, fs};

#[derive(ValueEnum, Clone, Debug)]
enum DriverLanguage {
    Python,
    Javascript,
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

    #[arg(short, long)]
    output_directory: PathBuf,
}

fn get_input(input_file: &str) -> (String, String) {
    (
        fs::read_to_string(input_file).expect("Unable to open file."),
        input_file.to_owned(),
    )
}

fn create_dfa_file(
    file_path: &PathBuf,
    dfa: &DFA,
    entry_state: &BTreeSet<NodeRef>,
    end_nodes: &HashMap<NodeRef, String>,
    token_order: &Vec<String>,
) {
    if let Some(parent_dir) = file_path.parent() {
        create_dir_all(parent_dir).expect("Unable to create parent directory.");
    }
    let mut file = File::create(file_path).unwrap();
    let json_string = serialize_dfa(dfa, entry_state, end_nodes, token_order);
    match file.write_all(json_string.as_bytes()) {
        Ok(_) => {}
        Err(why) => panic!("Error writing serialized DFA to JSON file: {}", why),
    };
}

fn create_driver_file(
    file_path: &PathBuf,
    token_order: Vec<String>,
    driver_generator: &dyn DriverGeneratable,
) {
    let user_defined_token_ids: Vec<String> = token_order
        .into_iter()
        .filter(|token| *token != "!")
        .collect();
    if let Some(parent_dir) = file_path.parent() {
        create_dir_all(parent_dir).expect("Unable to create parent directory.");
    }
    let mut driver_file = File::create(file_path).unwrap();
    let contents = generate_driver_content(driver_generator, &user_defined_token_ids);
    driver_file.write_all(contents.as_bytes()).unwrap();
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
    debug!("End nodes: {:?}", &parse_output.end_nodes);
    debug!("Node input symbols: {:?}", &parse_output.node_input_symbols);
    for (node_ref, node) in ast.get_pool().iter().enumerate() {
        debug!(" {:?} => {:?}", ObjRef(node_ref as u32), node);
    }

    // Get firstpos, lastpos for each node
    let root = ObjRef((ast.size() - 1) as u32);
    let meta = AST::get_meta(ast, root);

    // Get followpos
    let followpos = get_follow_pos(ast, &meta, root);
    debug!("Follow pos:");
    for (node_ref, node) in followpos.iter() {
        debug!("  {:?} => {:?}", node_ref, node);
    }

    // Get DFA transition table
    let dfa_table = get_dfa(
        ast,
        &meta,
        &followpos,
        root,
        &parse_output.node_input_symbols,
    );

    // Create DFA serialized file
    create_dfa_file(
        &args.output_directory.join("states.json"),
        &dfa_table,
        &meta.get(root).first_pos,
        &parse_output.end_nodes,
        &parse_output.token_order,
    );

    // Generate Driver file
    let (filename, generatable): (&str, &dyn DriverGeneratable) = match args.driver_language {
        DriverLanguage::Python => ("driver.py", &PythonDriverGenerator {}),
        DriverLanguage::Javascript => ("driver.js", &JavascriptDriverGenerator {}),
    };
    create_driver_file(
        &args.output_directory.join(filename),
        parse_output.token_order,
        generatable,
    )
}
