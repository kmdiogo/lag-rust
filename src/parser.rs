//use crate::lexer::{Lexer, Token, TokenEntry};
//
//struct Parser {
//    lexer: Lexer,
//    current_token: TokenEntry
//}
//
//impl Parser {
//    fn match_stmt_list(&mut self) -> bool {
//        if self.current_token.token == Token::EOI {
//            return true
//        }
//        if !Parser::match_stmt(self) {
//            return false
//        }
//        return Parser::match_stmt_list(self)
//    }
//    fn match_stmt(&mut self) -> bool {
//        return Parser::match_class_stmt(self) ||
//               Parser::match_token_stmt(self) ||
//               Parser::match_ignore_stmt(self);
//    }
//
//    fn match_class_stmt(&mut self) -> bool {
//
//        if let Some(cur) = self.current_token {
//            if cur.token == Token::Class {
//                return false
//            }
//        }
//        if (peekNextToken(true).first != Class) {
//            return false;
//        }
//        cur = getNextToken(file, true);
//
//        if (peekNextToken(true).first != Id) {
//            return false;
//        }
//        cur = getNextToken(file, true);
//        currentClass = cur.second;
//        classLookupTable[currentClass] = vector<char>();
//
//        if (peekNextToken(false).first != SetStart && peekNextToken(false).first != SetStartNegate) {
//            return false;
//        }
//        cur = getNextToken(file, false);
//
//        if (!matchCItemList()) {
//            return false;
//        }
//
//        //cur = getNextToken(file, false);
//
//        if (peekNextToken(false).first != SetEnd && peekNextToken(false).first != DashSetEnd) {
//            return false;
//        }
//
//        cur = getNextToken(file, false);
//        return true;
//    }
//
//    fn match_token_stmt(&mut self) -> bool {
//
//    }
//
//    fn match_ignore_stmt(&mut self) -> bool {
//
//    }
//}
//
//impl Parser {
//    fn parse(&mut self) {
//        self.current_token = self.lexer.get_token();
//        Parser::match_stmt_list(self);
//    }
//}