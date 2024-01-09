use std::char;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::Lines;
extern crate itertools;
use itertools::Itertools;

#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
struct TokenEntry {
    token: Token,
    lexeme: String,
    line: usize,
    col: usize,
}

/// States for the tokenizer finite state machine (FSM)
enum State {
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

struct Transition {
    next_state: Option<State>,
    return_token: Option<TokenEntry>,
}

struct Lexer {
    state: State,
    input: Vec<char>,
    current_line: usize,
    pos: usize,
}

impl Lexer {
    pub fn from_string(input: &str) -> Self {
        Self {
            state: State::Initial,
            input: input.chars().collect_vec(),
            current_line: 0,
            pos: 0,
        }
    }
    fn _create_token_entry(&self, token: Token, lexeme: &String) -> TokenEntry {
        TokenEntry {
            token,
            lexeme: lexeme.clone(),
            line: self.current_line,
            col: self.pos % (self.current_line + 1),
        }
    }
}

impl Iterator for Lexer {
    type Item = TokenEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let state = State::Initial;
        let mut lexeme = String::new();

        while self.pos < self.input.len() {
            let c = self.input.get(self.pos).unwrap();
            let lookahead = self.input.get(self.pos + 1);
            lexeme.push(c.clone());

            let transition_id_or_character = |_char: char| {
                if _char.is_alphabetic() {
                    Transition {
                        next_state: Some(State::Identifier),
                        return_token: None,
                    }
                } else {
                    Transition {
                        next_state: None,
                        return_token: Some(self._create_token_entry(Token::Character, &lexeme)),
                    }
                }
            };

            let transition = match self.state {
                State::Initial => match *c {
                    'c' => Transition {
                        next_state: Some(State::ClassC),
                        return_token: None,
                    },
                    't' => Transition {
                        next_state: Some(State::TokenT),
                        return_token: None,
                    },
                    'i' => Transition {
                        next_state: Some(State::IgnoreI),
                        return_token: None,
                    },
                    '[' => Transition {
                        next_state: Some(State::ClassC),
                        return_token: None,
                    },
                    '-' => Transition {
                        next_state: Some(State::Dash),
                        return_token: None,
                    },
                    '/' => Transition {
                        next_state: Some(State::ForwardSlash),
                        return_token: None,
                    },
                    ']' => Transition {
                        next_state: None,
                        return_token: Some(self._create_token_entry(Token::SetEnd, &lexeme)),
                    },
                    ']' => Transition {
                        next_state: None,
                        return_token: Some(self._create_token_entry(Token::SetEnd, &lexeme)),
                    },
                    '(' => Transition {
                        next_state: None,
                        return_token: Some(self._create_token_entry(Token::OpenParen, &lexeme)),
                    },
                    ')' => Transition {
                        next_state: None,
                        return_token: Some(self._create_token_entry(Token::CloseParen, &lexeme)),
                    },
                    '/' => Transition {
                        next_state: None,
                        return_token: Some(self._create_token_entry(Token::Slash, &lexeme)),
                    },
                    '*' => Transition {
                        next_state: None,
                        return_token: Some(self._create_token_entry(Token::Star, &lexeme)),
                    },
                    '+' => Transition {
                        next_state: None,
                        return_token: Some(self._create_token_entry(Token::Plus, &lexeme)),
                    },
                    '?' => Transition {
                        next_state: None,
                        return_token: Some(self._create_token_entry(Token::Question, &lexeme)),
                    },
                    '|' => Transition {
                        next_state: None,
                        return_token: Some(self._create_token_entry(Token::Pipe, &lexeme)),
                    },
                    _char => transition_id_or_character(_char),
                },

                State::ClassC => match *c {
                    'l' => Transition {
                        next_state: Some(State::ClassL),
                        return_token: None,
                    },
                    _char => transition_id_or_character(_char),
                },
                State::ClassL => match *c {
                    'a' => Transition {
                        next_state: Some(State::ClassA),
                        return_token: None,
                    },
                    _char => transition_id_or_character(_char),
                },
                State::ClassA => match *c {
                    's' => Transition {
                        next_state: Some(State::ClassS),
                        return_token: None,
                    },
                    _char => transition_id_or_character(_char),
                },
                State::ClassS => match *c {
                    's' => {
                        let mut trans = Transition {
                            next_state: Some(State::Initial),
                            return_token: Some(self._create_token_entry(Token::Class, &lexeme)),
                        };
                        if let Some(c) = lookahead {
                            if c.is_alphanumeric() {
                                trans = Transition {
                                    next_state: Some(State::Identifier),
                                    return_token: None,
                                }
                            }
                        }
                        trans
                    }
                    _char => transition_id_or_character(_char),
                },
                // 'token' matches
                State::TokenT => match *c {
                    'o' => Transition {
                        next_state: Some(State::TokenO),
                        return_token: None,
                    },
                    _char => transition_id_or_character(_char),
                },
                State::TokenO => match *c {},

                _ => return None,
            };

            // Perform actual state transition
            if let Some(ns) = transition.next_state {
                self.state = ns;
            }

            self.pos += 1;

            if transition.return_token.is_some() {
                return transition.return_token;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    #[test]
    fn test_initial_state_tokens() {
        let cases = [
            (
                "class alpha [a-zA-Z_]",
                TokenEntry {
                    col: 0,
                    token: Token::Class,
                    lexeme: "class".to_string(),
                    line: 0,
                },
            ),
            (
                "token Ident /[alpha]([alpha]|[digit])* /",
                TokenEntry {
                    col: 0,
                    token: Token::Token,
                    lexeme: "token".to_string(),
                    line: 0,
                },
            ),
            (
                "ignore /[whitespace]+/",
                TokenEntry {
                    col: 0,
                    token: Token::Ignore,
                    lexeme: "ignore".to_string(),
                    line: 0,
                },
            ),
        ];

        for (input, expected_token_entry) in cases {
            let mut lexer = Lexer::from_string(input);
            let token_result = lexer.next();
            assert_equal(token_result, Some(expected_token_entry));
        }
    }
}
