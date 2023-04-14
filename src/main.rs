use std::fs::File;
use std::io::BufReader;

mod tokenizer;

pub fn main() {
    let file = BufReader::new(File::open("input_path_HERE").expect("Unable to open file."));
    println!("Hello, world!");
}
