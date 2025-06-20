use crate::arena::{Arena, ObjRef};
use std::collections::HashSet;
use std::fmt;

pub type NodeRef = ObjRef;

#[derive(Debug)]
pub enum ParseTreeNode {
    Character(String),
    Id(String),
    Star { child: NodeRef },
    Plus { child: NodeRef },
    Question { child: NodeRef },
    Concat { left: NodeRef, right: NodeRef },
    Union { left: NodeRef, right: NodeRef },
}

impl fmt::Display for ParseTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseTreeNode::Character(s) => write!(f, "Character({})", s),
            ParseTreeNode::Id(s) => write!(f, "Id({})", s),
            ParseTreeNode::Star { child: _child } => write!(f, "Star"),
            ParseTreeNode::Plus { child: _child } => write!(f, "Plus"),
            ParseTreeNode::Question { child: _child } => write!(f, "Question"),
            ParseTreeNode::Concat {
                left: _left,
                right: _right,
            } => write!(f, "Concat"),
            ParseTreeNode::Union {
                left: _left,
                right: _right,
            } => write!(f, "Union"),
        }
    }
}

pub type ParseTree = Arena<ParseTreeNode>;

pub struct ParseTreeNodeMeta {
    nullable: bool,
    first_pos: HashSet<NodeRef>,
    last_pos: HashSet<NodeRef>,
}

pub type ParseTreeMeta = Arena<ParseTreeNodeMeta>;
pub fn get_parse_tree_meta(tree: &ParseTree, root: ObjRef) -> ParseTreeMeta {
    let mut meta: Vec<ParseTreeNodeMeta> = Vec::new();
    for i in 0..tree.size() {
        meta.push(ParseTreeNodeMeta {
            nullable: false,
            first_pos: HashSet::new(),
            last_pos: HashSet::new(),
        })
    }

    fn calculate_meta(node_ref: NodeRef, meta: &mut Vec<ParseTreeNodeMeta>, ast: &ParseTree) {
        let ast_node = ast.get(node_ref);
        match ast_node {
            ParseTreeNode::Character(s) | ParseTreeNode::Id(s) => {
                meta[node_ref.0 as usize] = ParseTreeNodeMeta {
                    last_pos: HashSet::from([node_ref]),
                    first_pos: HashSet::from([node_ref]),
                    nullable: false,
                };
            }
            ParseTreeNode::Union { left, right } => {
                calculate_meta(*left, meta, ast);
                calculate_meta((*right), meta, ast);
                let left_meta = &meta[left.0 as usize];
                let right_meta = &meta[right.0 as usize];
                meta[node_ref.0 as usize] = ParseTreeNodeMeta {
                    last_pos: left_meta
                        .last_pos
                        .union(&right_meta.last_pos)
                        .cloned()
                        .collect(),
                    first_pos: left_meta
                        .first_pos
                        .union(&right_meta.first_pos)
                        .cloned()
                        .collect(),
                    nullable: left_meta.nullable || right_meta.nullable,
                };
            }
            ParseTreeNode::Concat { left, right } => {
                calculate_meta(*left, meta, ast);
                calculate_meta((*right), meta, ast);
                let left_meta = &meta[left.0 as usize];
                let right_meta = &meta[right.0 as usize];
                let nullable = left_meta.nullable && right_meta.nullable;
                let first_pos = match left_meta.nullable {
                    true => left_meta
                        .first_pos
                        .union(&right_meta.first_pos)
                        .cloned()
                        .collect(),
                    false => left_meta.first_pos.clone(),
                };
                let last_pos = match right_meta.nullable {
                    true => right_meta
                        .last_pos
                        .union(&left_meta.last_pos)
                        .cloned()
                        .collect(),
                    false => right_meta.last_pos.clone(),
                };
                meta[node_ref.0 as usize] = ParseTreeNodeMeta {
                    first_pos,
                    last_pos,
                    nullable,
                }
            }
            node @ (ParseTreeNode::Star { child }
            | ParseTreeNode::Plus { child }
            | ParseTreeNode::Question { child, .. }) => {
                calculate_meta(*child, meta, ast);
                let child_meta = &meta[child.0 as usize];
                let nullable = match node {
                    ParseTreeNode::Star { child: _ } | ParseTreeNode::Question { child: _ } => true,
                    _ => false,
                };
                meta[node_ref.0 as usize] = ParseTreeNodeMeta {
                    last_pos: child_meta.last_pos.clone(),
                    first_pos: child_meta.first_pos.clone(),
                    nullable,
                }
            }
        }
    }

    calculate_meta(root, &mut meta, tree);
    ParseTreeMeta::from(meta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tree_meta() {
        // (a|b)*abb
        let mut ast = ParseTree::default();
        let a_char = ast.add(ParseTreeNode::Character("a".to_string()));
        let b_char = ast.add(ParseTreeNode::Character("b".to_string()));
        let union_node = ast.add(ParseTreeNode::Union {
            left: a_char,
            right: b_char,
        });
        let star_node = ast.add(ParseTreeNode::Star { child: union_node });
        let a2_char = ast.add(ParseTreeNode::Character("a".to_string()));
        let cat1_node = ast.add(ParseTreeNode::Concat {
            left: star_node,
            right: a2_char,
        });
        let b2_char = ast.add(ParseTreeNode::Character("b".to_string()));
        let cat2_node = ast.add(ParseTreeNode::Concat {
            left: cat1_node,
            right: b2_char,
        });
        let b3_char = ast.add(ParseTreeNode::Character("b".to_string()));
        let root = ast.add(ParseTreeNode::Concat {
            left: cat2_node,
            right: b3_char,
        });

        let meta = get_parse_tree_meta(&ast, root);
        let meta_pool = meta.get_pool();
        let root_meta = &meta_pool[root.0 as usize];
        assert_eq!(meta.size(), ast.size());
        assert_eq!(root_meta.nullable, false);
        assert_eq!(
            root_meta.first_pos,
            HashSet::from([a_char, b_char, a2_char])
        );
        assert_eq!(root_meta.last_pos, HashSet::from([b3_char]))
    }
}
