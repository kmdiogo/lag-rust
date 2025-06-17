use std::collections::{HashMap, HashSet};

use crate::lexer::{Lexer, LexerMode, Token, TokenEntry};

#[derive(Debug)]
pub struct ParserErr {
    pub message: String,
    pub token: TokenEntry,
}

#[derive(Debug)]
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
    ignore_parse_trees: Vec<ParseTreeNode>,
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
        let t = self.lexer.peek_token();
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

        let identifier = self.lexer.get_token();
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

        let regex_node = Parser::match_regex_stmt(self)?;
        self.token_parse_trees
            .insert(identifier.lexeme.clone(), regex_node);

        Ok(true)
    }

    fn match_ignore_stmt(&mut self) -> Result<bool, ParserErr> {
        if self.lexer.peek_token().token != Token::Ignore {
            return Ok(false);
        }
        self.lexer.get_token();

        let regex_node = Parser::match_regex_stmt(self)?;
        self.ignore_parse_trees.push(regex_node);

        Ok(true)
    }

    fn match_regex_stmt(&mut self) -> Result<ParseTreeNode, ParserErr> {
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

        let parse_tree_root = ParseTreeNode {
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

        let regex_end = self.lexer.get_token();
        if regex_end.token != Token::ForwardSlash {
            return Err(ParserErr {
                message: format!(
                    "Unexpected token: '{}'. Regex definitions must end with '/'.",
                    regex_end.lexeme
                ),
                token: regex_end,
            });
        }

        Ok(parse_tree_root)
    }

    fn match_regex(&mut self) -> Result<Option<ParseTreeNode>, ParserErr> {
        let node = match Parser::match_rterm(self)? {
            None => None,
            Some(rterm) => {
                if self.lexer.peek_token().token == Token::Pipe {
                    self.lexer.get_token();
                    Some(ParseTreeNode {
                        node_type: ParseTreeNodeType::Union,
                        left: Box::new(Some(rterm)),
                        right: Box::new(Parser::match_regex(self)?),
                        value: "".to_string(),
                    })
                } else {
                    Some(rterm)
                }
            }
        };
        Ok(node)
    }

    fn match_rterm(&mut self) -> Result<Option<ParseTreeNode>, ParserErr> {
        let node = match Parser::match_rclosure(self)? {
            None => None,
            Some(rclosure) => {
                let right = Parser::match_rterm(self)?;
                match right {
                    None => Some(rclosure),
                    Some(right_node) => Some(ParseTreeNode {
                        node_type: ParseTreeNodeType::Concat,
                        left: Box::new(Some(rclosure)),
                        right: Box::new(Some(right_node)),
                        value: "".to_string(),
                    }),
                }
            }
        };
        Ok(node)
    }

    fn match_rclosure(&mut self) -> Result<Option<ParseTreeNode>, ParserErr> {
        let node = match Parser::match_rfactor(self)? {
            None => None,
            Some(rfactor) => {
                let operator_node = match self.lexer.peek_token().token {
                    operator @ (Token::Star | Token::Plus | Token::Question) => {
                        self.lexer.get_token();
                        if operator == Token::Star {
                            Some(ParseTreeNodeType::Star)
                        } else if operator == Token::Plus {
                            Some(ParseTreeNodeType::Plus)
                        } else {
                            Some(ParseTreeNodeType::Question)
                        }
                    }
                    _ => None,
                };
                match operator_node {
                    Some(node_type) => Some(ParseTreeNode {
                        node_type: node_type,
                        left: Box::new(Some(rfactor)),
                        right: Box::new(None),
                        value: "".to_string(),
                    }),
                    None => Some(rfactor),
                }
            }
        };
        Ok(node)
    }

    fn match_rfactor(&mut self) -> Result<Option<ParseTreeNode>, ParserErr> {
        self.lexer.mode = LexerMode::Regex;
        let peek_token = self.lexer.peek_token();

        let node = match peek_token.token {
            Token::Characters => {
                assert_eq!(peek_token.lexeme.len(), 1, "Got unexpected multiple characters '{}' when lexer is in single char capture mode. This indicates a bug in the lexer.", peek_token.lexeme);
                self.lexer.get_token();
                Some(ParseTreeNode {
                    node_type: ParseTreeNodeType::Character,
                    left: Box::new(None),
                    right: Box::new(None),
                    value: peek_token.lexeme,
                })
            }
            Token::BracketOpen => {
                self.lexer.get_token();
                self.lexer.mode = LexerMode::Default;
                let id_token = self.lexer.get_token();

                let id_token = match id_token.token {
                    Token::Characters => {
                        if !self.class_lookup_table.contains_key(&id_token.lexeme) {
                            return Err(ParserErr {
                                message: format!(
                                    "Undefined class identifier '{}'",
                                    id_token.lexeme
                                ),
                                token: id_token,
                            });
                        }
                        let bracket_close_token = self.lexer.get_token();
                        match bracket_close_token.token {
                            Token::BracketClose => Some(ParseTreeNode {
                                node_type: ParseTreeNodeType::Id,
                                left: Box::new(None),
                                right: Box::new(None),
                                value: id_token.lexeme,
                            }),
                            _ => {
                                return Err(ParserErr {
                                    message: format!(
                                    "Unexpected token: '{}'. Regex definitions must end with '/'.",
                                    bracket_close_token.lexeme
                                ),
                                    token: bracket_close_token,
                                })
                            }
                        }
                    }
                    _ => None,
                };
                self.lexer.mode = LexerMode::Regex;
                id_token
            }
            Token::ParenOpen => {
                self.lexer.get_token();
                match Parser::match_regex(self)? {
                    None => None,
                    Some(inner_regex) => match self.lexer.peek_token().token {
                        Token::ParenClose => {
                            self.lexer.get_token();
                            Some(inner_regex)
                        }
                        _ => None,
                    },
                }
            }
            _ => None,
        };
        self.lexer.mode = LexerMode::Default;
        Ok(node)
    }
}

impl Parser {
    pub fn parse(&mut self) -> Result<bool, ParserErr> {
        Ok(Parser::match_stmt_list(self)?)
    }

    pub fn new(lexer: Lexer) -> Self {
        Self {
            class_lookup_table: HashMap::new(),
            token_parse_trees: HashMap::new(),
            ignore_parse_trees: Vec::new(),
            lexer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(env_logger::init);
    }

    fn inorder_traversal(root: &ParseTreeNode) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        fn helper(node: Option<&ParseTreeNode>, result: &mut Vec<String>) {
            if let Some(n) = node {
                helper((*n.left).as_ref(), result);
                result.push(format!("{:?}({})", n.node_type, n.value));
                helper((*n.right).as_ref(), result);
            }
        }
        helper(Some(root), &mut result);
        result
    }

    #[test]
    fn test_match_class_stmt() {
        setup();
        let lexer = Lexer::from_string("class alpha [a-cG-I_1-3z5]");
        let mut parser = Parser::new(lexer);

        let parse_result = parser.parse().unwrap();
        assert_eq!(parse_result, true);
        assert_eq!(parser.class_lookup_table.contains_key("alpha"), true);
        assert_eq!(
            *parser.class_lookup_table.get("alpha").unwrap(),
            HashSet::from(['a', 'b', 'c', 'G', 'H', 'I', '_', '1', '2', '3', 'z', '5'])
        );
    }

    #[test]
    fn test_token_stmt() {
        setup();
        let lexer = Lexer::from_string("token sample /(a|b)*/");
        let mut parser = Parser::new(lexer);

        let parse_result = parser.parse().unwrap();
        assert_eq!(parse_result, true);
        assert_eq!(parser.token_parse_trees.contains_key("sample"), true);
        let parse_tree = parser.token_parse_trees.get("sample").unwrap();
        assert_eq!(
            inorder_traversal(parse_tree),
            vec![
                "Character(a)",
                "Union()",
                "Character(b)",
                "Star()",
                "Concat()",
                "End()"
            ]
        )
    }

    #[test]
    fn test_regex_undefined_identifier() {
        setup();
        let lexer = Lexer::from_string("ignore /[id_that_doesnt_exist/");
        let mut parser = Parser::new(lexer);
        let parse_result = parser.parse();
        assert!(
            parse_result.is_err(),
            "Expected parsing to fail with a non-existent ID"
        );
        assert!(parse_result
            .unwrap_err()
            .message
            .contains("Undefined class identifier"));
    }

    #[test]
    fn test_regex_with_class() {
        setup();
        let input = "class alpha [a-zA-Z]
ignore /[alpha]+/
";
        let lexer = Lexer::from_string(input);
        let mut parser = Parser::new(lexer);
        let parse_result = parser.parse().unwrap();
        assert_eq!(parse_result, true);
        assert_eq!(parser.ignore_parse_trees.len(), 1);
        let parse_tree = &parser.ignore_parse_trees[0];
        assert_eq!(
            inorder_traversal(parse_tree),
            vec!["Id(alpha)", "Plus()", "Concat()", "End()"]
        )
    }

    #[test]
    fn test_ignore_stmt() {
        setup();
        let lexer = Lexer::from_string("ignore /(a+|b*) /");
        let mut parser = Parser::new(lexer);

        let parse_result = parser.parse().unwrap();
        assert_eq!(parse_result, true);
        assert_eq!(parser.ignore_parse_trees.len(), 1);
        let parse_tree = &parser.ignore_parse_trees[0];
        assert_eq!(
            inorder_traversal(parse_tree),
            vec![
                "Character(a)",
                "Plus()",
                "Union()",
                "Character(b)",
                "Star()",
                "Concat()",
                "Character( )",
                "Concat()",
                "End()"
            ]
        )
    }
}
