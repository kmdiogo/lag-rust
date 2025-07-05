import fs from "fs";
import path from "path";
import {Lexer} from "./driver.js"; // Adjust to the correct path of your Lexer module

function* inputGenerator() {
    const input = "abaaaacdb a";
    for (const c of input) {
        yield c;
    }
}

// Load DFA from states.json
const dfaPath = path.resolve("states.json");
const dfa = JSON.parse(fs.readFileSync(dfaPath, "utf8"));

// Instantiate the lexer and input generator
const lexer = new Lexer(dfa);
const gen = inputGenerator();

// Run lexer on input
for (let i = 0; i < 10; i++) {
    const token = lexer.getToken(gen);
    console.log(token.token, token.lexeme);
}
