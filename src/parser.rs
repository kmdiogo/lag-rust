use crate::lexer::Token::BracketClose;
use crate::lexer::{Lexer, LexerMode, Token, TokenEntry};
use crate::regex_ast::{NodeRef, ParseTree, ParseTreeNode};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ParserErr {
    pub message: String,
    pub token: TokenEntry,
}

#[derive(Debug)]
pub struct ParserOutput {
    class_lookup_table: HashMap<String, HashSet<char>>,
    token_parse_trees: HashMap<String, ParseTree>,
    ignore_parse_trees: Vec<ParseTree>,
}

struct ParserContext<'a> {
    lexer: &'a mut Lexer,
    class_lookup_table: &'a mut HashMap<String, HashSet<char>>,
    token_parse_trees: &'a mut HashMap<String, ParseTree>,
    ignore_parse_trees: &'a mut Vec<ParseTree>,
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

fn match_stmt_list(ctx: &mut ParserContext) -> Result<bool, ParserErr> {
    ctx.lexer.peek_token();
    if ctx.lexer.peek_token().token == Token::EOI {
        return Ok(true);
    }

    if !match_stmt(ctx)? {
        return Ok(false);
    }

    match_stmt_list(ctx)
}
fn match_stmt(ctx: &mut ParserContext) -> Result<bool, ParserErr> {
    Ok(match_class_stmt(ctx)? || match_token_stmt(ctx)? || match_ignore_stmt(ctx)?)
}

fn match_class_stmt(ctx: &mut ParserContext) -> Result<bool, ParserErr> {
    if ctx.lexer.peek_token().token != Token::Class {
        return Ok(false);
    }
    ctx.lexer.get_token();

    let identifier = ctx.lexer.peek_token();
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
    let current_class = ctx.lexer.get_token();
    ctx.class_lookup_table
        .insert(current_class.lexeme.to_string(), HashSet::new());

    let set_start = ctx.lexer.get_token();

    if set_start.token != Token::BracketOpen && set_start.token != Token::BracketOpenNegate {
        return Err(ParserErr {
            message: format!("Expected '[' but found {} instead", set_start.lexeme),
            token: set_start,
        });
    }

    let matched_c_item_list = match_c_item_list(ctx, &current_class.lexeme)?;
    if !matched_c_item_list {
        return Ok(false);
    }

    let set_end = ctx.lexer.get_token();
    if set_end.token != Token::BracketClose && set_end.token != Token::DashBracketClose {
        return Err(ParserErr {
            message: format!("Expected ']' but found {} instead", set_start.lexeme),
            token: set_end,
        });
    }

    return Ok(true);
}

fn match_c_item_list(ctx: &mut ParserContext, class_name: &str) -> Result<bool, ParserErr> {
    if ctx.lexer.peek_token().token == Token::BracketClose {
        return Ok(true);
    }

    if !match_c_item(ctx, class_name)? {
        return Ok(false);
    }

    match_c_item_list(ctx, class_name)
}

fn match_c_item(ctx: &mut ParserContext, class_name: &str) -> Result<bool, ParserErr> {
    let current_token = ctx.lexer.get_token();
    if current_token.token == Token::Characters {
        for ch in current_token.lexeme.chars() {
            // NOTE: unwrapping here since we know the previous parser match
            // guarantees the class_name will exist in the lookup table
            ctx.class_lookup_table
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
            ctx.class_lookup_table
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

fn match_token_stmt(ctx: &mut ParserContext) -> Result<bool, ParserErr> {
    if ctx.lexer.peek_token().token != Token::Token {
        return Ok(false);
    }
    ctx.lexer.get_token();

    let identifier = ctx.lexer.get_token();
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

    if let Some(regex_node) = match_regex_stmt(ctx)? {
        ctx.token_parse_trees
            .insert(identifier.lexeme.clone(), regex_node);
    }

    Ok(true)
}

fn match_ignore_stmt(ctx: &mut ParserContext) -> Result<bool, ParserErr> {
    if ctx.lexer.peek_token().token != Token::Ignore {
        return Ok(false);
    }
    ctx.lexer.get_token();

    let regex_node = match_regex_stmt(ctx)?;
    if let Some(node) = regex_node {
        ctx.ignore_parse_trees.push(node);
    }

    Ok(true)
}

fn match_regex_stmt(ctx: &mut ParserContext) -> Result<Option<ParseTree>, ParserErr> {
    let regex_begin = ctx.lexer.get_token();
    if regex_begin.token != Token::ForwardSlash {
        return Err(ParserErr {
            message: format!(
                "Unexpected token: '{}'. Regex definitions must start with '/'.",
                regex_begin.lexeme
            ),
            token: regex_begin,
        });
    }

    let mut tree = ParseTree::default();
    let regex_node = match_regex(ctx, &mut tree)?;
    match regex_node {
        Some(node) => {
            let end_node = tree.add(ParseTreeNode::Character('#'));
            tree.add(ParseTreeNode::Concat {
                left: node,
                right: end_node,
            });
        }
        None => return Ok(None),
    };

    let regex_end = ctx.lexer.get_token();
    if regex_end.token != Token::ForwardSlash {
        return Err(ParserErr {
            message: format!(
                "Unexpected token: '{}'. Regex definitions must end with '/'.",
                regex_end.lexeme
            ),
            token: regex_end,
        });
    }

    Ok(Some(tree))
}

fn match_regex(
    ctx: &mut ParserContext,
    tree: &mut ParseTree,
) -> Result<Option<NodeRef>, ParserErr> {
    let rterm_node = match match_rterm(ctx, tree)? {
        Some(n) => n,
        None => return Ok(None),
    };

    if ctx.lexer.peek_token().token != Token::Pipe {
        return Ok(Some(rterm_node));
    }

    let pipe_token = ctx.lexer.get_token();
    let right = match_regex(ctx, tree)?;
    let regex_node = match right {
        Some(right_node) => ParseTreeNode::Union {
            left: rterm_node,
            right: right_node,
        },
        None => {
            return Err(ParserErr {
                token: pipe_token,
                message: "Unexpected end of regex.".to_string(),
            })
        }
    };
    Ok(Some(tree.add(regex_node)))
}

fn match_rterm(
    ctx: &mut ParserContext,
    tree: &mut ParseTree,
) -> Result<Option<NodeRef>, ParserErr> {
    let rclosure_node = match match_rclosure(ctx, tree)? {
        Some(n) => n,
        None => return Ok(None),
    };

    let right = match_rterm(ctx, tree)?;
    let rterm_node = match right {
        Some(right_node) => ParseTreeNode::Concat {
            left: rclosure_node,
            right: right_node,
        },
        None => return Ok(Some(rclosure_node)),
    };
    Ok(Some(tree.add(rterm_node)))
}

fn match_rclosure(
    ctx: &mut ParserContext,
    tree: &mut ParseTree,
) -> Result<Option<NodeRef>, ParserErr> {
    let rfactor_node = match match_rfactor(ctx, tree)? {
        Some(n) => n,
        None => return Ok(None),
    };

    let operator_node = match ctx.lexer.peek_token().token {
        Token::Star => ParseTreeNode::Star {
            child: rfactor_node,
        },
        Token::Plus => ParseTreeNode::Plus {
            child: rfactor_node,
        },
        Token::Question => ParseTreeNode::Question {
            child: rfactor_node,
        },
        _ => return Ok(Some(rfactor_node)),
    };

    ctx.lexer.get_token();

    Ok(Some(tree.add(operator_node)))
}

fn match_rfactor(
    ctx: &mut ParserContext,
    tree: &mut ParseTree,
) -> Result<Option<NodeRef>, ParserErr> {
    ctx.lexer.mode = LexerMode::Regex;
    let peek_token = ctx.lexer.peek_token();
    ctx.lexer.mode = LexerMode::Default;

    let rfactor_node: NodeRef = match peek_token.token {
        Token::Characters => {
            assert_eq!(peek_token.lexeme.len(), 1, "Got unexpected multiple characters '{}' when lexer is in single char capture mode. This indicates a bug in the lexer.", peek_token.lexeme);
            ctx.lexer.mode = LexerMode::Regex;
            ctx.lexer.get_token();
            ctx.lexer.mode = LexerMode::Default;
            tree.add(ParseTreeNode::Character(
                peek_token.lexeme.chars().nth(0).unwrap(),
            ))
        }
        Token::BracketOpen => {
            ctx.lexer.get_token();
            let id_token = ctx.lexer.get_token();
            if id_token.token != Token::Characters {
                return Ok(None);
            }

            let char_set = match ctx.class_lookup_table.get(&id_token.lexeme) {
                Some(char_set) => char_set,
                None => {
                    return Err(ParserErr {
                        message: format!("Undefined class identifier '{}'", id_token.lexeme),
                        token: id_token,
                    })
                }
            };

            let bracket_close_token = ctx.lexer.get_token();
            if bracket_close_token.token != BracketClose {
                return Err(ParserErr {
                    message: format!(
                        "Expected closing ']' for character class reference. Got '{}' instead.",
                        bracket_close_token.lexeme
                    ),
                    token: bracket_close_token,
                });
            }

            ParseTree::add_charset(tree, char_set)
        }
        Token::ParenOpen => {
            ctx.lexer.get_token();
            let inner_regex_node = match match_regex(ctx, tree)? {
                Some(n) => n,
                None => return Ok(None),
            };

            let closing_paren_token = ctx.lexer.get_token();
            match closing_paren_token.token {
                Token::ParenClose => inner_regex_node,
                _ => {
                    return Err(ParserErr {
                        message: format!(
                            "Expected closing ')' for regex. Got '{}' instead.",
                            closing_paren_token.lexeme
                        ),
                        token: closing_paren_token,
                    })
                }
            }
        }
        _ => return Ok(None),
    };

    Ok(Some(rfactor_node))
}

pub fn parse(lexer: &mut Lexer) -> Result<ParserOutput, ParserErr> {
    let mut results = ParserOutput {
        class_lookup_table: HashMap::new(),
        token_parse_trees: HashMap::new(),
        ignore_parse_trees: Vec::new(),
    };
    let mut context = ParserContext {
        lexer,
        class_lookup_table: &mut results.class_lookup_table,
        token_parse_trees: &mut results.token_parse_trees,
        ignore_parse_trees: &mut results.ignore_parse_trees,
    };
    match_stmt_list(&mut context)?;
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::arena::ObjRef;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(env_logger::init);
    }

    fn inorder_traversal(tree: &ParseTree) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        fn helper(node_ref: NodeRef, result: &mut Vec<String>, tree: &ParseTree) {
            let node = tree.get(node_ref);
            match node {
                ParseTreeNode::Character(_c) => {
                    result.push(format!("{}", node));
                    return;
                }
                ParseTreeNode::Star { child }
                | ParseTreeNode::Question { child }
                | ParseTreeNode::Plus { child } => {
                    helper(*child, result, tree);
                    result.push(format!("{}", node))
                }
                ParseTreeNode::Concat { left, right } | ParseTreeNode::Union { left, right } => {
                    helper(*left, result, tree);
                    result.push(format!("{}", node));
                    helper(*right, result, tree);
                }
            };
        }
        helper(ObjRef((tree.size() - 1) as u32), &mut result, tree);
        result
    }

    #[test]
    fn test_match_class_stmt() {
        setup();
        let mut lexer = Lexer::from_string("class alpha [a-cG-I_1-3z5]");

        let parse_result = parse(&mut lexer);
        assert_eq!(parse_result.is_ok(), true);

        let output = parse_result.unwrap();
        assert_eq!(output.class_lookup_table.contains_key("alpha"), true);
        assert_eq!(
            *output.class_lookup_table.get("alpha").unwrap(),
            HashSet::from(['a', 'b', 'c', 'G', 'H', 'I', '_', '1', '2', '3', 'z', '5'])
        );
    }

    #[test]
    fn test_token_stmt() {
        setup();
        let mut lexer = Lexer::from_string("token sample /(a|b)*/");

        let parse_result = parse(&mut lexer);
        assert_eq!(parse_result.is_ok(), true);

        let output = parse_result.unwrap();
        assert_eq!(output.token_parse_trees.contains_key("sample"), true);
        let parse_tree = output.token_parse_trees.get("sample").unwrap();
        assert_eq!(
            inorder_traversal(parse_tree),
            vec![
                "Character(a)",
                "Union",
                "Character(b)",
                "Star",
                "Concat",
                "Character(#)"
            ]
        )
    }

    #[test]
    fn test_regex_undefined_identifier() {
        setup();
        let mut lexer = Lexer::from_string("ignore /[id_that_doesnt_exist/");
        let parse_result = parse(&mut lexer);
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
        let input = "class alpha [a-c]
ignore /[alpha]+/
";
        let mut lexer = Lexer::from_string(input);
        let parse_result = parse(&mut lexer);
        assert_eq!(parse_result.is_ok(), true);

        let output = parse_result.unwrap();
        assert_eq!(output.ignore_parse_trees.len(), 1);
        let parse_tree = &output.ignore_parse_trees[0];
        let possible_chars = vec!["Character(a)", "Character(b)", "Character(c)"];
        let inorder = inorder_traversal(parse_tree);
        for char_position in [0, 2, 4] {
            assert!(possible_chars.contains(&&*inorder[char_position]),)
        }
        for union_position in [1, 3] {
            assert_eq!(inorder[union_position], "Union")
        }
        assert_eq!(inorder[5..], vec!["Plus", "Concat", "Character(#)"])
    }

    #[test]
    fn test_ignore_stmt() {
        setup();
        let mut lexer = Lexer::from_string("ignore /(a+|b*) /");

        let parse_result = parse(&mut lexer);
        assert_eq!(parse_result.is_ok(), true);

        let output = parse_result.unwrap();
        assert_eq!(output.ignore_parse_trees.len(), 1);
        let parse_tree = &output.ignore_parse_trees[0];
        assert_eq!(
            inorder_traversal(parse_tree),
            vec![
                "Character(a)",
                "Plus",
                "Union",
                "Character(b)",
                "Star",
                "Concat",
                "Character( )",
                "Concat",
                "Character(#)"
            ]
        );
    }
}
