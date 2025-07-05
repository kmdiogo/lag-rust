use clap::Parser;
use std::fs;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_file: PathBuf,

    #[arg(short, long, default_value = "python")]
    driver_language: lag_rust_lib::DriverLanguage,

    #[arg(short, long)]
    output_directory: PathBuf,
}

pub fn main() {
    env_logger::init();
    let args = Args::parse();
    if !args.input_file.exists() {
        println!("Input file '{}' does not exist.", args.input_file.display());
        std::process::exit(1);
    }
    let text = fs::read_to_string(&args.input_file).expect("Unable to open file.");

    let lexer_program = match lag_rust_lib::generate_lexer_program(
        text.as_str(),
        &args.input_file.to_str().unwrap(),
        args.driver_language,
    ) {
        Ok(program) => program,
        Err(_err_message) => {
            println!("{}", _err_message);
            std::process::exit(1);
        }
    };

    println!("\nGenerating lexer files...");
    // Create DFA serialized file
    if !args.output_directory.exists() {
        create_dir_all(&args.output_directory).expect("Unable to create parent directory.");
    }
    let mut file = File::create(args.output_directory.join("states.json")).unwrap();
    match file.write_all(lexer_program.serialized_dfa_jsonstr.as_bytes()) {
        Ok(_) => {}
        Err(why) => panic!("Error writing serialized DFA to JSON file: {}", why),
    };

    // Generate Driver file
    let mut driver_file =
        File::create(args.output_directory.join(lexer_program.driver_filename)).unwrap();
    driver_file
        .write_all(lexer_program.driver_file_contents.as_bytes())
        .unwrap();
    println!("...Success!");
}
