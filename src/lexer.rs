use core::panic;
use std::char;
extern crate itertools;
use itertools::Itertools;

#[derive(Debug, PartialEq)]
enum Token {
    Class,
    Token,
    Id,
    Ignore,
    BracketOpen,
    BracketOpenNegate,
    BracketClose,
    DashBracketClose,
    ParenOpen,
    ParenClose,
    Pipe,
    Character,
    Dash,
    Star,
    Plus,
    Question,
    ForwardSlash,
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
#[derive(Debug)]
enum State {
    Initial,
    Identifier,
    Comment,
    // set token states
    BracketOpen,
    Dash,
    ForwardSlash,
    Escape,
}

struct Transition {
    next_state: Option<State>,
    return_token: Option<Token>,
}

struct Lexer {
    state: State,
    input: Vec<char>,
    current_line: usize,
    current_col: usize,
    pos: usize,
}

fn evaluate_escape_characters(lexeme: &String) -> String {
    match lexeme.as_str() {
        "\\n" => "\n".to_string(),
        "\\t" => "\t".to_string(),
        "\\f" => "\x0C".to_string(),
        "\\v" => "\x08".to_string(),
        "\\r" => "\r".to_string(),
        other => other[1..].to_string(),
    }
}

impl Lexer {
    pub fn from_string(input: &str) -> Self {
        Self {
            state: State::Initial,
            input: input.chars().collect_vec(),
            current_line: 0,
            current_col: 0,
            pos: 0,
        }
    }
    fn create_token_entry(&self, token: Token, lexeme: &String) -> TokenEntry {
        TokenEntry {
            token,
            lexeme: lexeme.clone(),
            line: self.current_line,
            col: self.current_col - lexeme.len(),
        }
    }
}

impl Iterator for Lexer {
    type Item = TokenEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut lexeme = String::new();

        while self.pos < self.input.len() {
            let c = self.input.get(self.pos).unwrap();
            let lookahead = self.input.get(self.pos + 1);
            lexeme.push(c.clone());

            let transition = match self.state {
                State::Initial => match *c {
                    ' ' => {
                        lexeme.clear();
                        Transition {
                            next_state: None,
                            return_token: None,
                        }
                    }
                    '-' => Transition {
                        next_state: Some(State::Dash),
                        return_token: None,
                    },
                    '/' => Transition {
                        next_state: Some(State::ForwardSlash),
                        return_token: None,
                    },
                    '[' => Transition {
                        next_state: Some(State::BracketOpen),
                        return_token: None,
                    },
                    ']' => Transition {
                        next_state: None,
                        return_token: Some(Token::BracketClose),
                    },
                    '(' => Transition {
                        next_state: None,
                        return_token: Some(Token::ParenOpen),
                    },
                    ')' => Transition {
                        next_state: None,
                        return_token: Some(Token::ParenClose),
                    },
                    '\\' => Transition {
                        next_state: Some(State::Escape),
                        return_token: None,
                    },
                    '*' => Transition {
                        next_state: None,
                        return_token: Some(Token::Star),
                    },
                    '+' => Transition {
                        next_state: None,
                        return_token: Some(Token::Plus),
                    },
                    '?' => Transition {
                        next_state: None,
                        return_token: Some(Token::Question),
                    },
                    '|' => Transition {
                        next_state: None,
                        return_token: Some(Token::Pipe),
                    },
                    _char => {
                        if _char.is_alphabetic() {
                            Transition {
                                next_state: Some(State::Identifier),
                                return_token: None,
                            }
                        } else {
                            Transition {
                                next_state: None,
                                return_token: Some(Token::Character),
                            }
                        }
                    }
                },
                State::Identifier => {
                    let done_with_identifier = match lookahead {
                        Some(lookahead_char) => !lookahead_char.is_alphanumeric(),
                        _ => true,
                    };

                    if done_with_identifier {
                        let token_entry = match lexeme.as_str() {
                            "class" => Token::Class,
                            "ignore" => Token::Ignore,
                            "token" => Token::Token,
                            _ => Token::Id,
                        };
                        Transition {
                            next_state: Some(State::Initial),
                            return_token: Some(token_entry),
                        }
                    } else {
                        Transition {
                            next_state: None,
                            return_token: None,
                        }
                    }
                }
                State::BracketOpen => match *c {
                    '^' => Transition {
                        next_state: Some(State::Initial),
                        return_token: Some(Token::BracketOpenNegate),
                    },
                    _ => {
                        self.pos -= 1;
                        self.current_col -= 1;
                        lexeme.pop();
                        Transition {
                            next_state: Some(State::Initial),
                            return_token: Some(Token::BracketOpen),
                        }
                    }
                },
                State::Escape => Transition {
                    next_state: Some(State::Initial),
                    return_token: Some(Token::Character),
                },
                State::ForwardSlash => match *c {
                    '/' => Transition {
                        next_state: Some(State::Comment),
                        return_token: None,
                    },
                    _ => {
                        self.pos -= 1;
                        self.current_col -= 1;
                        lexeme.pop();
                        Transition {
                            next_state: None,
                            return_token: Some(Token::ForwardSlash),
                        }
                    }
                },
                State::Comment => match *c {
                    // Consume characters until end of comment line
                    '\n' => {
                        lexeme.clear();
                        Transition {
                            next_state: Some(State::Initial),
                            return_token: None,
                        }
                    }
                    _ => Transition {
                        next_state: None,
                        return_token: None,
                    },
                },
                State::Dash => match *c {
                    ']' => Transition {
                        next_state: Some(State::Initial),
                        return_token: Some(Token::DashBracketClose),
                    },
                    _ => {
                        self.pos -= 1;
                        self.current_col -= 1;
                        lexeme.pop();
                        Transition {
                            next_state: Some(State::Initial),
                            return_token: Some(Token::Dash),
                        }
                    }
                },
            };

            // Perform actual state transition
            if let Some(ns) = transition.next_state {
                self.state = ns;
            }

            if let Some('\n') = self.input.get(self.pos) {
                self.current_line += 1;
                self.current_col = 0;
            } else {
                self.current_col += 1;
            }
            self.pos += 1;

            if let Some(rt) = transition.return_token {
                println!("Returning token: {:?}", rt);
                if rt == Token::Character {
                    let escaped_lexeme = evaluate_escape_characters(&lexeme);
                    return Some(self.create_token_entry(rt, &escaped_lexeme));
                }
                return Some(self.create_token_entry(rt, &lexeme));
            }
            println!(
                "State: {:?}, Line: {}, Col: {}",
                self.state, self.current_line, self.current_col
            );
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;

    #[test]
    fn test_cpp_identifier_definition() {
        let input = "class alpha [a-zA-z]
class digit [0-9]
class whitespace [\\n\\t\\f\\v\\r ]

token Ident /[alpha]([alpha]|[digit])* /
ignore /[whitespace]+/
";
        let input2 = "class alpha [a-zA-z]";
        let mut lexer = Lexer::from_string(input2);
        let result_tokens = lexer.collect_vec();
        let expected_tokens = vec![
            TokenEntry {
                col: 0,
                token: Token::Class,
                lexeme: "class".to_string(),
                line: 0,
            },
            TokenEntry {
                col: 6,
                token: Token::Id,
                lexeme: "alpha".to_string(),
                line: 0,
            },
            TokenEntry {
                col: 12,
                token: Token::BracketOpen,
                lexeme: "[".to_string(),
                line: 0,
            },
        ];
        assert_eq!(result_tokens, expected_tokens)
    }

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

    #[test]
    fn test_escape_characters() {
        let cases = [
            ("//this is a long comment that should be ignore\n\\n", "\n"),
            ("\\t", "\t"),
            ("\\f", "\x0C"),
            ("\\v", "\x08"),
            ("\\r", "\r"),
            ("\\%", "%"),
        ];

        for (input, expected_lexeme) in cases {
            let mut lexer = Lexer::from_string(input);
            let token_result = lexer.next();
            let expected_token_entry = TokenEntry {
                col: 0,
                line: 0,
                lexeme: expected_lexeme.to_string(),
                token: Token::Character,
            };
            assert_equal(token_result, Some(expected_token_entry));
        }
    }
}
