use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeRef(u32);

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

#[derive(Debug)]
pub struct ParseTree(Vec<ParseTreeNode>);

impl ParseTree {
    /// Create an empty pool.
    pub(crate) fn default() -> Self {
        Self(Vec::new())
    }

    /// Dereference an AST node reference, obtaining the underlying `ParseTreeNode`.
    pub fn get(&self, node_ref: NodeRef) -> &ParseTreeNode {
        &self.0[node_ref.0 as usize]
    }

    pub fn get_root_ref(&self) -> Option<NodeRef> {
        if self.0.len() == 0 {
            return None;
        }
        Some(NodeRef((self.0.len() - 1) as u32))
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Add a node to the tree and get a reference to it.
    pub fn add(&mut self, node: ParseTreeNode) -> NodeRef {
        let idx = self.0.len();
        self.0.push(node);
        NodeRef(idx.try_into().expect("too many exprs in the pool"))
    }
}

pub struct ParseTreeNodeMeta {
    nullable: bool,
    first_pos: HashSet<NodeRef>,
    last_pos: HashSet<NodeRef>,
}

pub fn getParseTreeMeta(tree: &ParseTree) -> Vec<ParseTreeNodeMeta> {
    let mut meta: Vec<ParseTreeNodeMeta> = Vec::new();
    for i in 0..tree.0.len() {
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
                    true => left_meta
                        .last_pos
                        .union(&right_meta.last_pos)
                        .cloned()
                        .collect(),
                    false => left_meta.last_pos.clone(),
                };
                meta[node_ref.0 as usize] = ParseTreeNodeMeta {
                    first_pos,
                    last_pos,
                    nullable,
                }
            }
            ParseTreeNode::Star { child } => {
                calculate_meta(*child, meta, ast);
                let child_meta = &meta[child.0 as usize];
                meta[node_ref.0 as usize] = ParseTreeNodeMeta {
                    last_pos: child_meta.last_pos.clone(),
                    first_pos: child_meta.first_pos.clone(),
                    nullable: true,
                }
            }
            ParseTreeNode::Plus { child } | ParseTreeNode::Question { child } => {
                calculate_meta(*child, meta, ast);
                let child_meta = &meta[child.0 as usize];
                meta[node_ref.0 as usize] = ParseTreeNodeMeta {
                    last_pos: child_meta.last_pos.clone(),
                    first_pos: child_meta.first_pos.clone(),
                    nullable: false,
                }
            }
        }
    }

    meta
}
