use std::collections::HashMap;

use crate::lexer::{Lexer, Token, TokenEntry};

struct Parser {
    lexer: Lexer,
    current_token: TokenEntry,
    class_lookup_table: HashMap<String, Vec<char>>,
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
    fn match_stmt_list(&mut self) -> bool {
        if self.lexer.peek_token().token == Token::EOI {
            return true;
        }
        if !Parser::match_stmt(self) {
            return false;
        }
        return Parser::match_stmt_list(self);
    }
    fn match_stmt(&mut self) -> bool {
        return Parser::match_class_stmt(self)
            || Parser::match_token_stmt(self)
            || Parser::match_ignore_stmt(self);
    }

    fn match_class_stmt(&mut self) -> bool {
        if self.lexer.peek_token().token != Token::Class {
            return false;
        }
        self.lexer.get_token();

        let lookahead = self.lexer.peek_token();
        if lookahead.token != Token::Characters || !is_identifier(&lookahead.lexeme) {
            return Error();
            return false;
        }

        let current_class_name = self.current_token.lexeme;
        self.class_lookup_table
            .insert(current_class_name, Vec::new());

        self.current_token = self.lexer.get_token();

        if self.current_token.token != Token::BracketOpen
            && self.current_token.token != Token::BracketOpenNegate
        {
            return false;
        }

        self.current_token = self.lexer.get_token();

        if !Parser::match_c_item_list(self, &current_class_name) {
            return false;
        }

        if self.current_token.token != Token::BracketClose
            && self.current_token.token != Token::DashBracketClose
        {
            return false;
        }

        self.current_token = self.lexer.get_token();
        return true;
    }

    fn match_c_item_list(&mut self, class_name: &str) -> bool {
        if !Parser::match_c_item(self, class_name) {
            return true;
        }
        return Parser::match_c_item_list(self, class_name);
    }

    fn match_c_item(&mut self, class_name: &str) -> bool {
        self.current_token = self.lexer.get_token();
        if self.current_token.token != Token::Characters {
            return false;
        }

        if self.lexer.peek_token().token == Token::Dash {}
    }

    fn match_token_stmt(&mut self) -> bool {
        return true;
    }

    fn match_ignore_stmt(&mut self) -> bool {
        return true;
    }
}

impl Parser {
    fn parse(&mut self) {
        Parser::match_stmt_list(self);
    }
}
