//! Parsing logic for lexical analyzer generator
//! Implements recursive descent parser for input file grammar with some
//! side effects to update lookup tables

use crate::arena::ObjRef;
use crate::lexer::Token::BracketClose;
use crate::lexer::{Lexer, LexerMode, Token, TokenEntry};
use crate::regex_ast::{ASTNode, NodeRef, AST};
use log::debug;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug)]
pub struct ParserErr {
    pub message: String,
    pub token: TokenEntry,
}

#[derive(Debug)]
pub enum ClassSetOperator {
    Include,
    Negate,
}

#[derive(Debug)]
pub struct ClassSetEntry {
    pub chars: HashSet<char>,
    pub operator: ClassSetOperator,
}

#[derive(Debug)]
pub struct ParserOutput {
    class_lookup_table: BTreeMap<String, ClassSetEntry>,
    pub node_input_symbols: HashMap<NodeRef, HashSet<char>>,
    pub token_order: Vec<String>,
    pub end_nodes: HashMap<NodeRef, String>,
    pub tree: AST,
}

struct ParserContext<'a> {
    lexer: &'a mut Lexer,
    tree: &'a mut AST,
    leaf_nodes: &'a mut Vec<NodeRef>,
    token_order: &'a mut Vec<String>,
    accepting_nodes: &'a mut HashMap<NodeRef, String>,
    class_lookup_table: &'a mut BTreeMap<String, ClassSetEntry>,
}

fn is_identifier(lexeme: &str) -> bool {
    if lexeme.is_empty() {
        return false;
    }

    let mut chars = lexeme.chars();
    let first_char = chars.next().unwrap();
    if !(first_char.is_ascii_alphabetic() || first_char == '_') {
        return false;
    }

    for ch in chars {
        if !(ch.is_ascii_alphanumeric() || ch == '_') {
            return false;
        }
    }

    return true;
}

/// stmtList → stmt stmtList
///          | ε
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

/// stmt → classStmt
///      | tokenStmt
///      | ignoreStmt
fn match_stmt(ctx: &mut ParserContext) -> Result<bool, ParserErr> {
    Ok(match_class_stmt(ctx)? || match_token_stmt(ctx)? || match_ignore_stmt(ctx)?)
}

/// classStmt → class id [ cltemList ]
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

    let set_start = ctx.lexer.get_token();
    let class_set_operator = match set_start.token {
        Token::BracketOpen => ClassSetOperator::Include,
        Token::BracketOpenNegate => ClassSetOperator::Negate,
        _ => {
            return Err(ParserErr {
                message: format!("Expected '[' but found {} instead", set_start.lexeme),
                token: set_start,
            });
        }
    };

    ctx.class_lookup_table.insert(
        current_class.lexeme.to_string(),
        ClassSetEntry {
            chars: HashSet::new(),
            operator: class_set_operator,
        },
    );

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

    Ok(true)
}

/// cItemList → cItem cItemList
/// | ε
fn match_c_item_list(ctx: &mut ParserContext, class_name: &str) -> Result<bool, ParserErr> {
    if ctx.lexer.peek_token().token == Token::BracketClose {
        return Ok(true);
    }

    if !match_c_item(ctx, class_name)? {
        return Ok(false);
    }

    match_c_item_list(ctx, class_name)
}

/// cItem → <any_char>              * ] must be escaped
///       | <any_char> - <any_char>
fn match_c_item(ctx: &mut ParserContext, class_name: &str) -> Result<bool, ParserErr> {
    let current_token = ctx.lexer.get_token();
    if current_token.token == Token::Characters {
        for ch in current_token.lexeme.chars() {
            // NOTE: unwrapping here since we know the previous parser match
            // guarantees the class_name will exist in the lookup table
            ctx.class_lookup_table
                .get_mut(class_name)
                .unwrap()
                .chars
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
                .chars
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

fn add_regex_subtree(ctx: &mut ParserContext, token_id: String) -> Result<(), ParserErr> {
    // Check if tree already exists in AST
    let left_node = match ctx.tree.size() > 0 {
        true => Some(ObjRef((ctx.tree.size() - 1) as u32)),
        false => None,
    };
    let regex_node = match_regex_stmt(ctx, &token_id)?;
    if let Some(node) = regex_node {
        if let Some(left) = left_node {
            ctx.tree.add(ASTNode::Union { left, right: node });
        }
    }

    Ok(())
}

/// tokenStmt → token <chars> / regexStmt /
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

    ctx.token_order.push(identifier.lexeme.clone());
    add_regex_subtree(ctx, identifier.lexeme)?;

    Ok(true)
}

/// ignoreStmt → ignore regexStmt
fn match_ignore_stmt(ctx: &mut ParserContext) -> Result<bool, ParserErr> {
    if ctx.lexer.peek_token().token != Token::Ignore {
        return Ok(false);
    }
    ctx.lexer.get_token();

    add_regex_subtree(ctx, "!".to_string())?;

    Ok(true)
}

/// regexStmt → / regex /
fn match_regex_stmt(
    ctx: &mut ParserContext,
    token_id: &String,
) -> Result<Option<NodeRef>, ParserErr> {
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

    let regex_node = match_regex(ctx)?;
    let regex_root = match regex_node {
        Some(node) => {
            let end_node = ctx.tree.add(ASTNode::Character('#'));
            ctx.accepting_nodes.insert(end_node, token_id.clone());
            Some(ctx.tree.add(ASTNode::Concat {
                left: node,
                right: end_node,
            }))
        }
        None => None,
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

    Ok(regex_root)
}

/// regex → regex | rTerm
///       | rTerm
fn match_regex(ctx: &mut ParserContext) -> Result<Option<NodeRef>, ParserErr> {
    let rterm_node = match match_rterm(ctx)? {
        Some(n) => n,
        None => return Ok(None),
    };

    if ctx.lexer.peek_token().token != Token::Pipe {
        return Ok(Some(rterm_node));
    }

    let pipe_token = ctx.lexer.get_token();
    let right = match_regex(ctx)?;
    let regex_node = match right {
        Some(right_node) => ASTNode::Union {
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
    Ok(Some(ctx.tree.add(regex_node)))
}

/// rTerm → rTerm rClosure
///       | rClosure
fn match_rterm(ctx: &mut ParserContext) -> Result<Option<NodeRef>, ParserErr> {
    let rclosure_node = match match_rclosure(ctx)? {
        Some(n) => n,
        None => return Ok(None),
    };

    let right = match_rterm(ctx)?;
    let rterm_node = match right {
        Some(right_node) => ASTNode::Concat {
            left: rclosure_node,
            right: right_node,
        },
        None => return Ok(Some(rclosure_node)),
    };
    Ok(Some(ctx.tree.add(rterm_node)))
}

/// rClosure → rFactor *
///          | rFactor +
///          | rFactor ?
///          | rFactor
fn match_rclosure(ctx: &mut ParserContext) -> Result<Option<NodeRef>, ParserErr> {
    let rfactor_node = match match_rfactor(ctx)? {
        Some(n) => n,
        None => return Ok(None),
    };

    let operator_node = match ctx.lexer.peek_token().token {
        Token::Star => ASTNode::Star {
            child: rfactor_node,
        },
        Token::Plus => ASTNode::Plus {
            child: rfactor_node,
        },
        Token::Question => ASTNode::Question {
            child: rfactor_node,
        },
        _ => return Ok(Some(rfactor_node)),
    };

    ctx.lexer.get_token();

    Ok(Some(ctx.tree.add(operator_node)))
}

/// rFactor → character
///         | [ id ]
///         | ( regex )
fn match_rfactor(ctx: &mut ParserContext) -> Result<Option<NodeRef>, ParserErr> {
    ctx.lexer.mode = LexerMode::Regex;
    let peek_token = ctx.lexer.peek_token();
    ctx.lexer.mode = LexerMode::Default;

    let rfactor_node: NodeRef = match peek_token.token {
        Token::Characters => {
            assert_eq!(peek_token.lexeme.len(), 1, "Got unexpected multiple characters '{}' when lexer is in single char capture mode. This indicates a bug in the lexer.", peek_token.lexeme);
            ctx.lexer.mode = LexerMode::Regex;
            ctx.lexer.get_token();
            ctx.lexer.mode = LexerMode::Default;
            let node_char = peek_token.lexeme.chars().nth(0).unwrap();
            if !node_char.is_ascii() {
                return Err(ParserErr {
                    token: peek_token,
                    message: format!("Unsupported character '{}' in regex expression.", node_char),
                });
            }
            let node = ctx.tree.add(ASTNode::Character(node_char));
            ctx.leaf_nodes.push(node);
            node
        }
        Token::BracketOpen => {
            ctx.lexer.get_token();
            let id_token = ctx.lexer.get_token();
            if id_token.token != Token::Characters {
                return Ok(None);
            }

            if !ctx.class_lookup_table.contains_key(&id_token.lexeme) {
                return Err(ParserErr {
                    message: format!("Undefined class identifier '{}'", id_token.lexeme),
                    token: id_token,
                });
            }

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

            let node = ctx.tree.add(ASTNode::Id(id_token.lexeme));
            ctx.leaf_nodes.push(node);
            node
        }
        Token::ParenOpen => {
            ctx.lexer.get_token();
            let inner_regex_node = match match_regex(ctx)? {
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

/// Adds the disjoint set of characters to the alphabet lookup table
/// Overlapping characters from different character sets will be grouped into 1
fn get_disjoint_alphabet(
    terminal_nodes: &Vec<NodeRef>,
    tree: &AST,
    class_lookup_table: &BTreeMap<String, ClassSetEntry>,
) -> HashMap<char, HashSet<NodeRef>> {
    let ascii_chars: HashSet<char> = (0u8..=127).map(|b| b as char).collect();
    let mut alphabet: HashMap<char, HashSet<NodeRef>> = HashMap::new();
    for node_ref in terminal_nodes {
        let node = tree.get(*node_ref);
        match node {
            ASTNode::Character(c) => {
                alphabet
                    .entry(*c)
                    .or_insert(HashSet::new())
                    .insert(*node_ref);
            }
            ASTNode::Id(id) => {
                let class_set = class_lookup_table.get(id).unwrap_or_else(|| {
                    panic!("ID node {:?} could not be found in class table", node)
                });
                let class_chars = match class_set.operator {
                    ClassSetOperator::Include => &class_set.chars,
                    ClassSetOperator::Negate => &(&ascii_chars - &class_set.chars),
                };
                for char in class_chars {
                    alphabet
                        .entry(*char)
                        .or_insert(HashSet::new())
                        .insert(*node_ref);
                }
            }
            _ => panic!(
                "Encountered non-terminal node {:?} when trying to compute disjoint class set",
                node
            ),
        }
    }
    alphabet
}

/// Gets each node's related input symbols
fn get_node_input_symbols(
    alphabet: &HashMap<char, HashSet<NodeRef>>,
) -> HashMap<NodeRef, HashSet<char>> {
    let mut node_input_symbols = HashMap::new();
    for (char, node_refs) in alphabet {
        for node_ref in node_refs {
            node_input_symbols
                .entry(*node_ref)
                .or_insert(HashSet::new())
                .insert(*char);
        }
    }

    node_input_symbols
}

/// Parse a stream of input tokens into an AST + other useful metadata
pub fn parse(lexer: &mut Lexer) -> Result<ParserOutput, ParserErr> {
    let mut class_lookup_table = BTreeMap::new();
    let mut end_nodes = HashMap::new();
    let mut tree = AST::default();
    let mut token_order = Vec::new();
    let mut terminal_nodes: Vec<_> = Vec::new();

    let mut ctx = ParserContext {
        lexer,
        class_lookup_table: &mut class_lookup_table,
        accepting_nodes: &mut end_nodes,
        tree: &mut tree,
        token_order: &mut token_order,
        leaf_nodes: &mut terminal_nodes,
    };
    match_stmt_list(&mut ctx)?;
    debug!("Terminal nodes: {:?}", ctx.leaf_nodes);

    // Add ignore character
    ctx.token_order.push("!".to_string());
    // Add end char to AST
    if ctx.tree.size() > 0 {
        let left = ObjRef((ctx.tree.size() - 1) as u32);
        let end_char = ctx.tree.add(ASTNode::Character('#'));
        ctx.tree.add(ASTNode::Concat {
            left,
            right: end_char,
        });
    }

    let alphabet = get_disjoint_alphabet(&terminal_nodes, &tree, &class_lookup_table);
    debug!("Alphabet: {:?}", &alphabet);
    let mut node_input_symbols = get_node_input_symbols(&alphabet);
    // Add end nodes (technically have '#' as input symbol
    for (end_node, _) in &end_nodes {
        node_input_symbols.insert(*end_node, HashSet::from(['#']));
    }

    Ok(ParserOutput {
        class_lookup_table,
        end_nodes,
        tree,
        token_order,
        node_input_symbols,
    })
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

    fn inorder_traversal(tree: &AST) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        fn helper(node_ref: NodeRef, result: &mut Vec<String>, tree: &AST) {
            let node = tree.get(node_ref);
            match node {
                ASTNode::Character(_) | ASTNode::Id(_) => {
                    result.push(format!("{}", node));
                    return;
                }
                ASTNode::Star { child } | ASTNode::Question { child } | ASTNode::Plus { child } => {
                    helper(*child, result, tree);
                    result.push(format!("{}", node))
                }
                ASTNode::Concat { left, right } | ASTNode::Union { left, right } => {
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
    fn test_disjoint_alphabet() {
        setup();
        let mut ast = AST::default();
        let regex1_node = ast.add(ASTNode::Id("regex1".to_string()));
        let regex2_node = ast.add(ASTNode::Id("regex2".to_string()));
        let a_node = ast.add(ASTNode::Character('a'));
        let class_lookup_table: BTreeMap<String, ClassSetEntry> = BTreeMap::from([
            (
                "regex1".to_string(),
                ClassSetEntry {
                    operator: ClassSetOperator::Include,
                    chars: HashSet::from(['a', 'b', 'c', 'd']),
                },
            ),
            (
                "regex2".to_string(),
                ClassSetEntry {
                    operator: ClassSetOperator::Negate,
                    chars: HashSet::from(['b']),
                },
            ),
        ]);
        let alphabet = get_disjoint_alphabet(
            &vec![regex1_node, regex2_node, a_node],
            &ast,
            &class_lookup_table,
        );

        let mut expected_alphabet: HashMap<char, HashSet<NodeRef>> = HashMap::from([
            ('a', HashSet::from([a_node, regex1_node, regex2_node])),
            ('b', HashSet::from([regex1_node])),
            ('c', HashSet::from([regex1_node, regex2_node])),
            ('d', HashSet::from([regex1_node, regex2_node])),
        ]);
        let ascii_chars: HashSet<char> = (0u8..=127).map(|b| b as char).collect();
        for char in ascii_chars {
            if expected_alphabet.contains_key(&char) {
                continue;
            }
            expected_alphabet.insert(char, HashSet::from([regex2_node]));
        }

        for (char, node_refs) in expected_alphabet.iter() {
            assert!(
                alphabet.contains_key(char),
                "Alphabet does not contain char '{}'",
                char
            );
            assert_eq!(
                alphabet.get(char).unwrap(),
                node_refs,
                "Alphabet char '{}' does not contain all expected node refs",
                char
            );
        }
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
            output.class_lookup_table.get("alpha").unwrap().chars,
            HashSet::from(['a', 'b', 'c', 'G', 'H', 'I', '_', '1', '2', '3', 'z', '5'])
        );
        assert!(matches!(
            output.class_lookup_table.get("alpha").unwrap().operator,
            ClassSetOperator::Include {}
        ));
    }

    #[test]
    fn test_token_stmt() {
        setup();
        let mut lexer = Lexer::from_string("token sample /(a|b)*/");

        let parse_result = parse(&mut lexer);
        assert_eq!(parse_result.is_ok(), true);

        let output = parse_result.unwrap();
        let parse_tree = output.tree;
        assert_eq!(
            inorder_traversal(&parse_tree),
            vec![
                "Character(a)",
                "Union",
                "Character(b)",
                "Star",
                "Concat",
                "Character(#)",
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
        let parse_tree = &output.tree;
        let inorder = inorder_traversal(parse_tree);
        assert_eq!(
            inorder,
            vec![
                "Id(alpha)",
                "Plus",
                "Concat",
                "Character(#)",
                "Concat",
                "Character(#)"
            ]
        );
    }

    #[test]
    fn test_ignore_stmt() {
        setup();
        let mut lexer = Lexer::from_string("ignore /(a+|b*) /");

        let parse_result = parse(&mut lexer);
        assert_eq!(parse_result.is_ok(), true);

        let output = parse_result.unwrap();
        let parse_tree = &output.tree;
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
                "Character(#)",
                "Concat",
                "Character(#)"
            ]
        );
    }
}
