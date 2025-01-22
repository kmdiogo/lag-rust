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
        if self.current_token.token == Token::EOI {
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
        if self.current_token.token != Token::Class {
            return false;
        }

        self.current_token = self.lexer.get_token();

        if self.current_token.token != Token::Characters
            || !is_identifier(&self.current_token.lexeme)
        {
            return false;
        }

        self.class_lookup_table
            .insert(self.current_token.lexeme, Vec::new());

        self.current_token = self.lexer.get_token();

        if self.current_token.token != Token::BracketOpen
            && self.current_token.token != Token::BracketOpenNegate
        {
            return false;
        }

        self.current_token = self.lexer.get_token();

        if !Parser::match_c_item_list(self) {
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

    fn match_token_stmt(&mut self) -> bool {}

    fn match_ignore_stmt(&mut self) -> bool {}
}

impl Parser {
    fn parse(&mut self) {
        self.current_token = self.lexer.get_token();
        Parser::match_stmt_list(self);
    }
}

