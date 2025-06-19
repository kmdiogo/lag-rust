use std::collections::{HashMap, HashSet};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParseTreeNode {
    Character(String),
    Id(String),
    Star {
        child: Box<ParseTreeNode>,
    },
    Plus {
        child: Box<ParseTreeNode>,
    },
    Question {
        child: Box<ParseTreeNode>,
    },
    Concat {
        left: Box<ParseTreeNode>,
        right: Box<ParseTreeNode>,
    },
    Union {
        left: Box<ParseTreeNode>,
        right: Box<ParseTreeNode>,
    },
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

pub struct ParseTreeNodeMeta {
    nullable: bool,
    first_pos: HashSet<Uuid>,
    last_pos: HashSet<Uuid>,
}

pub struct ParseTreeMeta<'a> {
    node_ids: HashMap<&'a ParseTreeNode, Uuid>,
    meta: HashMap<Uuid, ParseTreeNodeMeta>,
}

impl ParseTreeMeta<'_> {
    fn get_node_meta(&mut self, node: &ParseTreeNode) -> Option<&ParseTreeNodeMeta> {
        let node_id = match self.node_ids.get(node) {
            Some(id) => id,
            None => return None,
        };
        
        self.meta.get(node_id)
    }
    fn assign_node_ids(&mut self, root: &ParseTreeNode) -> &ParseTreeNodeMeta {
        let node = root;
        if let Some(cached_meta) = self.get_node_meta(node) {
            return cached_meta;
        }
        let node_id = Uuid::new_v4();
        self.node_ids.insert(node, node_id);
        match node {
            ParseTreeNode::Character(s) | ParseTreeNode::Id(s) => {
                &self.meta.insert(node_id, ParseTreeNodeMeta {
                    last_pos: HashSet::from([node_id]),
                    first_pos: HashSet::from([node_id]),
                    nullable: false
                }).unwrap();
                self.meta.get(&node_id).unwrap()
            },
            ParseTreeNode::Union {left, right} => {
                let left_meta = self.assign_node_ids((*left).as_ref());
                let right_meta = self.assign_node_ids((*right).as_ref());
                &self.meta.insert(node_id, ParseTreeNodeMeta {
                    last_pos: left_meta.last_pos.union(&right_meta.last_pos).cloned().collect(),
                    first_pos: left_meta.first_pos.union(&right_meta.first_pos).cloned().collect(),
                    nullable: left_meta.nullable || right_meta.nullable
                }).unwrap()
            },
            ParseTreeNode::Concat {left, right} => {
                let left_meta = self.assign_node_ids((*left).as_ref());
                let right_meta = self.assign_node_ids((*right).as_ref());
                let nullable = left_meta.nullable && right_meta.nullable;
                let first_pos: HashSet<Uuid> = match left_meta.nullable {
                    true => left_meta.first_pos.union(&right_meta.first_pos).cloned().collect(),
                    false => left_meta.first_pos.clone()
                };
                let last_pos: HashSet<Uuid> = match right_meta.nullable {
                    true => left_meta.last_pos.union(&right_meta.last_pos).cloned().collect(),
                    false => left_meta.last_pos.clone()
                };
                &self.meta.insert(node_id, ParseTreeNodeMeta {
                    first_pos,
                    last_pos,
                    nullable
                }).unwrap()
            },
            ParseTreeNode::Star { child } => {
                let child_meta = self.assign_node_ids((*child).as_ref());
                &self.meta.insert(node_id, ParseTreeNodeMeta {
                    last_pos: child_meta.last_pos.clone(),
                    first_pos: child_meta.first_pos.clone(),
                    nullable: true 
                }).unwrap()
            },
            ParseTreeNode::Plus { child } | ParseTreeNode::Question { child } => {
                let child_meta = self.assign_node_ids((*child).as_ref());
                &self.meta.insert(node_id, ParseTreeNodeMeta {
                    last_pos: child_meta.last_pos.clone(),
                    first_pos: child_meta.first_pos.clone(),
                    nullable: false 
                }).unwrap()
            }
        }

    }
    pub fn get_ast_meta(&mut self) {
        let helper = |node: &ParseTreeNode| {
            if let Some(id) = self.node_ids.get(&node) {
                if self.meta.contains_key(id) {
                    return self.meta.get(id).unwrap();
                }
            }
            let node_id = self.node_ids.get(&node);
            if self.meta.contains_key(node_id) {
                return meta[node];
            }
            match node {
                ParseTreeNode::Character(s) | ParseTreeNode::Id(s) => {
                    meta.insert(
                        node,
                        ParseTreeMeta {
                            last_pos: HashSet::from([node]),
                            first_pos: HashSet::from([node]),
                            nullable: false,
                        },
                    );
                }
                ParseTreeNode::Star { child } => {
                    helper(&*child, meta);
                    meta.insert(
                        node,
                        ParseTreeMeta {
                            last_pos: meta.get(child).last_pos,
                        },
                    );
                }
            }
        }
    }
}
