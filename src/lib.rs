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
use crate::regex_ast::{get_dfa, get_follow_pos, AST};
use clap::ValueEnum;
use log::debug;
use std::fmt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(ValueEnum, Clone, Debug)]
pub enum DriverLanguage {
    Python,
    Javascript,
}

impl fmt::Display for DriverLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self) // or use a custom string if needed
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct LexerProgram {
    pub driver_file_contents: String,
    pub driver_filename: String,
    pub serialized_dfa_jsonstr: String,
}

#[wasm_bindgen]
pub fn generate_lexer_program(
    input_text: &str,
    input_filepath: &str,
    driver_language: DriverLanguage,
) -> Result<LexerProgram, String> {
    let mut lexer = Lexer::from_string(input_text);

    println!("Parsing...");
    let parse_output = match parse(&mut lexer) {
        Ok(output) => {
            println!("...Success!");
            output
        }
        Err(e) => {
            return Err(format!(
                "Parsing error. See below for details: \n\t{}\n\t{}:{}:{}",
                e.message, input_filepath, e.token.line, e.token.col
            ));
        }
    };
    let ast = &parse_output.tree;
    if ast.size() == 0 {
        return Err("No token definitions found.".to_string());
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

    println!("\nGenerating lexer state machine...");
    // Get DFA transition table
    let dfa_table = get_dfa(
        ast,
        &meta,
        &followpos,
        root,
        &parse_output.node_input_symbols,
    );
    println!("...Success!");

    // Get JSON representation of DFA transition table
    let serialized_dfa_jsonstr = serialize_dfa(
        &dfa_table,
        &meta.get(root).first_pos,
        &parse_output.end_nodes,
        &parse_output.token_order,
    );

    // Get driver file text content
    let (driver_filename, generatable): (&str, &dyn DriverGeneratable) = match driver_language {
        DriverLanguage::Python => ("driver.py", &PythonDriverGenerator {}),
        DriverLanguage::Javascript => ("driver.js", &JavascriptDriverGenerator {}),
    };
    let user_defined_token_ids: Vec<String> = parse_output
        .token_order
        .into_iter()
        .filter(|token| *token != "!")
        .collect();
    let driver_file_contents = generate_driver_content(generatable, &user_defined_token_ids);

    Ok(LexerProgram {
        driver_file_contents,
        serialized_dfa_jsonstr,
        driver_filename: driver_filename.to_string(),
    })
}
