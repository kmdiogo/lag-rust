use crate::tokenizer::TokenizerState::TokenE;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::Lines;

enum Token {
    Class,
    Token,
    Id,
    Ignore,
    SetStart,
    SetStartNegate,
    SetEnd,
    DashSetEnd,
    OpenParen,
    CloseParen,
    Slash,
    Pipe,
    Character,
    Dash,
    Star,
    Plus,
    Question,
    EOI,
}

/// Token + metadata parser uses
struct TokenEntry {
    token: Token,
    lexeme: String,
    line: usize,
    col: usize,
}

/// States for the tokenizer finite state machine (FSM)
enum TokenizerState {
    Initial,
    // 'class' keyword states
    ClassC,
    ClassL,
    ClassA,
    ClassS,
    // 'token' keyword states
    TokenT,
    TokenO,
    TokenK,
    TokenE,
    TokenN,
    // 'ignore' keyword states
    IgnoreI,
    IgnoreG,
    IgnoreN,
    IgnoreO,
    IgnoreR,
    Identifier,
    // set token states
    BracketOpen,
    Dash,
    ForwardSlash,
}

struct Tokenizer {
    state: TokenizerState,
    input: Vec<String>,
    current_line: usize,
    current_col: usize,
}

impl Tokenizer {
    pub fn from_string(input: &String) -> Self {
        Self {
            state: TokenizerState::Initial,
            input: input.lines().collect(),
            current_line: 0,
            current_col: 0,
        }
    }
    fn _create_token_entry(&self, token: Token, lexeme: String) -> TokenEntry {
        TokenEntry {
            token,
            lexeme,
            line: self.current_line,
            col: self.current_col,
        }
    }
}

impl Iterator for Tokenizer {
    fn next(&mut self) {
        let mut lexeme = String::new();
        while self.current_line < self.input.len() {
            let line = &self.input[self.current_line];
            while self.current_col < line.len() {
                let c = &line.chars().nth(self.current_col).unwrap();
                if self.state == TokenizerState::Initial {
                    lexeme = String::new();
                }
                lexeme.push(c.clone());
                match self.state {
                    TokenizerState::Initial => {
                        if c == 'c' {
                            self.state = TokenizerState::ClassC
                        } else if c == 't' {
                            self.state = TokenizerState::TokenT
                        } else if c == 'i' {
                            self.state = TokenizerState::IgnoreI
                        } else if c == '[' {
                            self.state = TokenizerState::BracketOpen
                        } else if c == '-' {
                            self.state = TokenizerState::Dash
                        } else if c == '/' {
                            self.state = TokenizerState::ForwardSlash
                        } else if c == ']' {
                            return TokenEntry {
                                token: Token::SetEnd,
                            };
                        } else if c == '(' {
                            return TokenEntry {
                                token: Token::OpenParen,
                            };
                        } else if c == ')' {
                            return TokenEntry {
                                token: Token::CloseParen,
                            };
                        } else if c == '/' {
                            return TokenEntry {
                                token: Token::Slash,
                            };
                        } else if c == '*' {
                            return TokenEntry { token: Token::Star };
                        } else if c == '+' {
                            return TokenEntry { token: Token::Plus };
                        } else if c == '?' {
                            return TokenEntry {
                                token: Token::Question,
                            };
                        } else if c == '|' {
                            return TokenEntry { token: Token::Pipe };
                        } else if c.is_alphanumeric() {
                            self.state = TokenizerState::Identifier
                        }
                    }

                    TokenizerState::ClassC => {
                        if c == 'l' {
                            self.state = TokenizerState::ClassL
                        } else if c.is_alphanumeric() {
                            self.state = TokenizerState::Identifier
                        } else {
                            return TokenEntry {
                                token: Token::Character,
                            };
                        }
                    }
                    TokenizerState::ClassL => {
                        if c == 'a' {
                            self.state = TokenizerState::ClassA
                        } else if c.is_alphanumeric() {
                            self.state = TokenizerState::Identifier
                        } else {
                            return TokenEntry {
                                token: Token::Character,
                            };
                        }
                    }
                    TokenizerState::ClassA => {
                        if c == 's' {
                            self.state = TokenizerState::ClassS
                        } else if c.is_alphanumeric() {
                            self.state = TokenizerState::Identifier
                        } else {
                            return TokenEntry {
                                token: Token::Character,
                            };
                        }
                    }
                    TokenizerState::ClassS => {
                        if c == 's' {
                            return TokenEntry {
                                token: Token::Class,
                            };
                        } else if c.is_alphanumeric() {
                            self.state = TokenizerState::Identifier
                        } else {
                            return TokenEntry {
                                token: Token::Character,
                            };
                        }
                    }
                }

                self.current_col += 1;
            }

            self.current_line += 1;
        }
    }
}
