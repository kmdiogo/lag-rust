# lag-rust 🦀  
Lexical Analyzer Generator (LAG) re-written in Rust (blazingly fast btw 🔥)

## 📘 Overview  
`lag-rust` is a tool for generating lexical analyzers (lexers) from token definitions. 
It parses input files describing tokens, constructs a deterministic finite automaton (DFA), and outputs both a serialized DFA table (in JSON 📄) and a driver program for tokenizing input (selectable in Python or JavaScript).

## ✨ Features  
- Fast lexer generation using Rust ⚡  
- Output driver code in Python or JavaScript
- Produces a JSON file with the DFA for integration or debugging 
- WASM bindings for web usage 🌐  

## ⚙️ Usage  

### 🖥️ CLI  
Compile and run the tool using Cargo 📦:  
```bash
cargo build --release  
./target/release/lag-rust --input-file <path/to/your/token_definitions.txt> --output-directory <output/dir> --driver-language <python|javascript>
```

🔧 CLI Options:
- `--input-file` / `-i`: Path to the input file containing token definitions  
- `--output-directory` / `-o`: Directory where output files will be written  
- `--driver-language` / `-d`: Target language for the generated driver (`python` or `javascript`) — defaults to `python`

### 📌 Example  
```bash
./target/release/lag-rust -i tokens.txt -o ./out -d javascript
```

📤 This will generate:  
- `out/states.json` (serialized DFA)  
- `out/driver.js` (driver code for lexing)  

## 📚 Library  
You can use `lag-rust` as a library or via WASM 🕸️. The main API entrypoint is:  
```rust
lag_rust_lib::generate_lexer_program(input_text, input_filepath, driver_language)
```

📦 A pre-built WASM library is published to npm:
- [npmjs.com/package/lag_rust](https://www.npmjs.com/package/lag_rust  )

## 📋 Requirements  
- Rust (edition 2021)

## 🛠️ Development  
Clone the repo and build it yourself:  
```bash
git clone https://github.com/kmdiogo/lag-rust.git  
cd lag-rust  
cargo build
```

## 🔗 Repository  
🌐 [GitHub – kmdiogo/lag-rust](https://github.com/kmdiogo/lag-rust)
