use std::collections::{HashMap, HashSet};

use crate::lexer::{Lexer, Token, TokenEntry};

#[derive(Debug)]
pub struct ParserErr {
    pub message: String,
    pub token: TokenEntry,
}

pub enum ParseTreeNodeType {
    Concat,
    Character,
    Id,
    Plus,
    Question,
    Star,
    Union,
    End,
}

pub struct ParseTreeNode {
    pub node_type: ParseTreeNodeType,
    pub left: Box<Option<ParseTreeNode>>,
    pub right: Box<Option<ParseTreeNode>>,
    pub value: String,
}

pub struct Parser {
    lexer: Lexer,
    class_lookup_table: HashMap<String, HashSet<char>>,
    token_parse_trees: HashMap<String, ParseTreeNode>,
}

fn is_identifier(lexeme: &str) -> bool {
    if lexeme.is_empty() {
        return false;
    }

    let mut chars = lexeme.chars();
    if !chars.next().unwrap().is_alphabetic() {
        return false;
    }

    for ch in chars {
        if !ch.is_alphanumeric() {
            return false;
        }
    }

    return true;
}

impl Parser {
    fn match_stmt_list(&mut self) -> Result<bool, ParserErr> {
        if self.lexer.peek_token().token == Token::EOI {
            return Ok(true);
        }

        if !Parser::match_stmt(self)? {
            return Ok(false);
        }

        Parser::match_stmt_list(self)
    }
    fn match_stmt(&mut self) -> Result<bool, ParserErr> {
        Ok(Parser::match_class_stmt(self)?
            || Parser::match_token_stmt(self)?
            || Parser::match_ignore_stmt(self)?)
    }

    fn match_class_stmt(&mut self) -> Result<bool, ParserErr> {
        if self.lexer.peek_token().token != Token::Class {
            return Ok(false);
        }
        self.lexer.get_token();

        let identifier = self.lexer.peek_token();
        if identifier.token != Token::Characters {
            return Err(ParserErr {
                message: format!("Unexpected token: '{}'", identifier.lexeme),
                token: identifier,
            });
        }

        if !is_identifier(&identifier.lexeme) {
            return Err(ParserErr {
                message: format!("Invalid class identifier: '{}'", identifier.lexeme),
                token: identifier,
            });
        }

        // Initialize character vector for this defined class that will hold the character set
        let current_class = self.lexer.get_token();
        self.class_lookup_table
            .insert(current_class.lexeme.to_string(), HashSet::new());

        let set_start = self.lexer.get_token();

        if set_start.token != Token::BracketOpen && set_start.token != Token::BracketOpenNegate {
            return Err(ParserErr {
                message: format!("Expected '[' but found {} instead", set_start.lexeme),
                token: set_start,
            });
        }

        let matched_c_item_list = Parser::match_c_item_list(self, &current_class.lexeme)?;
        if !matched_c_item_list {
            return Ok(false);
        }

        let set_end = self.lexer.get_token();
        if set_end.token != Token::BracketClose && set_end.token != Token::DashBracketClose {
            return Err(ParserErr {
                message: format!("Expected ']' but found {} instead", set_start.lexeme),
                token: set_end,
            });
        }

        return Ok(true);
    }

    fn match_c_item_list(&mut self, class_name: &str) -> Result<bool, ParserErr> {
        if self.lexer.peek_token().token == Token::BracketClose {
            return Ok(true);
        }

        if !Parser::match_c_item(self, class_name)? {
            return Ok(false);
        }

        Parser::match_c_item_list(self, class_name)
    }

    fn match_c_item(&mut self, class_name: &str) -> Result<bool, ParserErr> {
        let current_token = self.lexer.get_token();
        if current_token.token == Token::Characters {
            for ch in current_token.lexeme.chars() {
                // NOTE: unwrapping here since we know the previous parser match
                // guarantees the class_name will exist in the lookup table
                self.class_lookup_table
                    .get_mut(class_name)
                    .unwrap()
                    .insert(ch);
            }
        } else if current_token.token == Token::CharacterRange {
            let mut char_iter = current_token.lexeme.chars();
            // We want a panic here if any of characters are null (implies an error in the Lexer)
            let range_start = char_iter.next().unwrap();
            let _dash = char_iter.next().unwrap();
            let range_end = char_iter.next().unwrap();
            if range_start as u32 > range_end as u32 {
                return Err(ParserErr {
                    message: format!("Invalid character range '{}'. Starting character must come before the end character", current_token.lexeme),
                    token: current_token,
                });
            }
            for i in range_start as u32..range_end as u32 + 1 {
                // Unwrapping here since i is derived from a char and guaranteed to work here
                self.class_lookup_table
                    .get_mut(class_name)
                    .unwrap()
                    .insert(char::from_u32(i).unwrap());
            }
        } else {
            return Err(ParserErr {
                message: format!("Unexpected token: {}", current_token.lexeme),
                token: current_token,
            });
        }

        Ok(true)
    }

    fn match_token_stmt(&mut self) -> Result<bool, ParserErr> {
        if self.lexer.peek_token().token != Token::Token {
            return Ok(false);
        }
        self.lexer.get_token();

        let identifier = self.lexer.peek_token();
        if identifier.token != Token::Characters {
            return Err(ParserErr {
                message: format!("Unexpected token: '{}'", identifier.lexeme),
                token: identifier,
            });
        }

        if !is_identifier(&identifier.lexeme) {
            return Err(ParserErr {
                message: format!("Invalid token identifier: '{}'", identifier.lexeme),
                token: identifier,
            });
        }

        let regex_begin = self.lexer.get_token();
        if regex_begin.token != Token::ForwardSlash {
            return Err(ParserErr {
                message: format!(
                    "Unexpected token: '{}'. Regex definitions must start with '/'.",
                    regex_begin.lexeme
                ),
                token: regex_begin,
            });
        }

        let mut parse_tree_root = ParseTreeNode {
            left: Box::new(Parser::match_regex(self)?),
            right: Box::new(Some(ParseTreeNode {
                node_type: ParseTreeNodeType::End,
                left: Box::new(None),
                right: Box::new(None),
                value: "".to_string(),
            })),
            node_type: ParseTreeNodeType::Concat,
            value: "".to_string(),
        };

        let regex_end = self.lexer.peek_token();
        if regex_end.token != Token::ForwardSlash {
            return Err(ParserErr {
                message: format!(
                    "Unexpected token: '{}'. Regex definitions must end with '/'.",
                    regex_end.lexeme
                ),
                token: regex_end,
            });
        }

        Ok(true)
    }

    fn match_regex(&mut self) -> Result<Option<ParseTreeNode>, ParserErr> {
        if Parser::match_rterm(self)?.is_none() {
            return Ok(None);
        }

        while self.lexer.peek_token().token == Token::Pipe {}
    }

    fn match_rterm(&mut self) -> Result<Option<ParseTreeNode>, ParserErr> {}

    fn match_rclosure(&mut self) -> Result<ParseTreeNode, ParserErr> {}

    fn match_rfactor(&mut self) -> Result<ParseTreeNode, ParserErr> {}

    fn match_ignore_stmt(&mut self) -> Result<bool, ParserErr> {
        return Ok(true);
    }
}

impl Parser {
    pub fn parse(&mut self) -> Result<bool, ParserErr> {
        Ok(Parser::match_stmt_list(self)?)
    }

    pub fn new(lexer: Lexer) -> Self {
        Self {
            class_lookup_table: HashMap::new(),
            lexer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_class_stmt() {
        env_logger::init();
        let lexer = Lexer::from_string("class alpha [a-cG-I_1-3z5]");
        let mut parser = Parser {
            class_lookup_table: HashMap::new(),
            lexer,
        };

        let parse_result = parser.parse().unwrap();
        assert_eq!(parse_result, true);
        assert_eq!(parser.class_lookup_table.contains_key("alpha"), true);
        assert_eq!(
            *parser.class_lookup_table.get("alpha").unwrap(),
            HashSet::from(['a', 'b', 'c', 'G', 'H', 'I', '_', '1', '2', '3', 'z', '5'])
        );
    }
}
