use crate::arena::{Arena, ObjRef};
use log::debug;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt;

pub type NodeRef = ObjRef;

#[derive(Debug)]
pub enum ASTNode {
    Character(char),
    Id(String),
    Star { child: NodeRef },
    Plus { child: NodeRef },
    Question { child: NodeRef },
    Concat { left: NodeRef, right: NodeRef },
    Union { left: NodeRef, right: NodeRef },
}

impl fmt::Display for ASTNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ASTNode::Character(char) => {
                write!(f, "Character({})", char)
            }
            ASTNode::Id(id) => {
                write!(f, "Id({})", id)
            }
            ASTNode::Star { child: _child } => write!(f, "Star"),
            ASTNode::Plus { child: _child } => write!(f, "Plus"),
            ASTNode::Question { child: _child } => write!(f, "Question"),
            ASTNode::Concat {
                left: _left,
                right: _right,
            } => write!(f, "Concat"),
            ASTNode::Union {
                left: _left,
                right: _right,
            } => write!(f, "Union"),
        }
    }
}

pub type AST = Arena<ASTNode>;

pub struct ASTMetaNode {
    pub nullable: bool,
    pub first_pos: BTreeSet<NodeRef>,
    pub last_pos: BTreeSet<NodeRef>,
}

pub type ASTMeta = Arena<ASTMetaNode>;

impl AST {
    /// Computes nullable(i), firstpos(i), lastpos(i) for all nodes i in an AST.
    /// Returns a separate tree whose indices in its underlying object pool are parallel to the provided AST.
    pub fn get_meta(tree: &AST, root: NodeRef) -> ASTMeta {
        let mut meta: Vec<ASTMetaNode> = Vec::new();
        for _i in 0..tree.size() {
            meta.push(ASTMetaNode {
                nullable: false,
                first_pos: BTreeSet::new(),
                last_pos: BTreeSet::new(),
            })
        }

        fn calculate_meta(node_ref: NodeRef, meta: &mut Vec<ASTMetaNode>, ast: &AST) {
            let ast_node = ast.get(node_ref);
            match ast_node {
                ASTNode::Character(_) | ASTNode::Id(_) => {
                    meta[node_ref.0 as usize] = ASTMetaNode {
                        last_pos: BTreeSet::from([node_ref]),
                        first_pos: BTreeSet::from([node_ref]),
                        nullable: false,
                    };
                }
                ASTNode::Union { left, right } => {
                    calculate_meta(*left, meta, ast);
                    calculate_meta(*right, meta, ast);
                    let left_meta = &meta[left.0 as usize];
                    let right_meta = &meta[right.0 as usize];
                    meta[node_ref.0 as usize] = ASTMetaNode {
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
                ASTNode::Concat { left, right } => {
                    calculate_meta(*left, meta, ast);
                    calculate_meta(*right, meta, ast);
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
                    meta[node_ref.0 as usize] = ASTMetaNode {
                        first_pos,
                        last_pos,
                        nullable,
                    }
                }
                node @ (ASTNode::Star { child }
                | ASTNode::Plus { child }
                | ASTNode::Question { child, .. }) => {
                    calculate_meta(*child, meta, ast);
                    let child_meta = &meta[child.0 as usize];
                    let nullable = match node {
                        ASTNode::Star { child: _ } | ASTNode::Question { child: _ } => true,
                        _ => false,
                    };
                    meta[node_ref.0 as usize] = ASTMetaNode {
                        last_pos: child_meta.last_pos.clone(),
                        first_pos: child_meta.first_pos.clone(),
                        nullable,
                    }
                }
            }
        }

        calculate_meta(root, &mut meta, tree);
        ASTMeta::from(meta)
    }
}

/// Computes all followpos(i), for all nodes i in an AST
pub fn get_follow_pos(
    ast: &AST,
    meta: &ASTMeta,
    root: NodeRef,
) -> HashMap<NodeRef, BTreeSet<NodeRef>> {
    let end_node = match ast.get(root) {
        ASTNode::Concat { left: _, right} => *right,
        _ => panic!("Root node is not a concatenation. Root of parse tree must be a cat node with right child of '#'"),
    };
    let mut follow_pos: HashMap<NodeRef, BTreeSet<NodeRef>> =
        HashMap::from([(end_node, BTreeSet::new())]);
    fn helper(
        ast: &AST,
        meta: &ASTMeta,
        node_ref: NodeRef,
        follow_pos: &mut HashMap<NodeRef, BTreeSet<NodeRef>>,
    ) {
        let node = meta.get(node_ref);
        match ast.get(node_ref) {
            ASTNode::Concat { left, right, .. } => {
                let follow_nodes = &meta.get(*right).first_pos;
                for nref in &meta.get(*left).last_pos {
                    let nref_followpos = follow_pos.entry(*nref).or_insert(BTreeSet::new());
                    nref_followpos.extend(follow_nodes.into_iter())
                }
                helper(ast, meta, *left, follow_pos);
                helper(ast, meta, *right, follow_pos);
            }
            ASTNode::Star { child } | ASTNode::Plus { child } => {
                let follow_nodes = &node.first_pos;
                for nref in &node.last_pos {
                    let nref_followpos = follow_pos.entry(*nref).or_insert(BTreeSet::new());
                    nref_followpos.extend(follow_nodes.into_iter())
                }
                helper(ast, meta, *child, follow_pos);
            }
            ASTNode::Union { left, right } => {
                helper(ast, meta, *left, follow_pos);
                helper(ast, meta, *right, follow_pos);
            }
            _ => {}
        };
    }
    helper(ast, meta, root, &mut follow_pos);
    follow_pos
}

/// Gets all nodes in a state set grouped by input character
fn group_nodes_by_input_symbol(
    node_input_symbols: &HashMap<NodeRef, HashSet<char>>,
    node_refs: &BTreeSet<NodeRef>,
) -> HashMap<String, BTreeSet<NodeRef>> {
    let mut groups: HashMap<String, BTreeSet<NodeRef>> = HashMap::new();
    for node_ref in node_refs {
        for char in node_input_symbols.get(node_ref).unwrap() {
            groups
                .entry(char.to_string())
                .or_insert(BTreeSet::new())
                .insert(*node_ref);
        }
    }
    groups
}

pub struct DFA {
    pub state_table: HashMap<BTreeSet<NodeRef>, HashMap<String, BTreeSet<NodeRef>>>,
}

/// Computes the DFA transition table from an AST
pub fn get_dfa(
    ast: &AST,
    meta: &ASTMeta,
    followpos: &HashMap<NodeRef, BTreeSet<NodeRef>>,
    root: NodeRef,
    node_input_symbols: &HashMap<NodeRef, HashSet<char>>,
) -> DFA {
    let root_meta = meta.get(root);
    let mut accepting_states = HashSet::new();
    let mut state_table: HashMap<BTreeSet<NodeRef>, HashMap<String, BTreeSet<NodeRef>>> =
        HashMap::from([(root_meta.first_pos.clone(), HashMap::new())]);
    let mut queue: VecDeque<BTreeSet<NodeRef>> = VecDeque::from([root_meta.first_pos.clone()]);
    while queue.len() > 0 {
        let cur_state = queue.pop_front().unwrap();
        debug!("State {:?}", cur_state);
        let grouped_nodes = group_nodes_by_input_symbol(node_input_symbols, &cur_state);
        for (input_symbol, node_refs) in grouped_nodes {
            if input_symbol == "#" {
                accepting_states.insert(cur_state.clone());
                continue;
            }
            debug!("  {} groups => {:?}", input_symbol, node_refs);
            let mut target_state: BTreeSet<NodeRef> = BTreeSet::new();
            for node_ref in &node_refs {
                debug!(
                    "Getting followpos for node {:?}({:?})",
                    node_ref,
                    ast.get(*node_ref)
                );
                target_state = target_state
                    .union(followpos.get(&node_ref).unwrap())
                    .cloned()
                    .collect();
            }
            debug!("    target: {:?}", target_state);
            if !state_table.contains_key(&target_state) {
                state_table.insert(target_state.clone(), HashMap::new());
                queue.push_back(target_state.clone());
            }
            let state_transition = state_table
                .entry(cur_state.clone())
                .or_insert(HashMap::new());
            state_transition.insert(input_symbol, target_state.clone());
        }
    }
    DFA { state_table }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tree_meta() {
        // (a|b)*abb
        let mut ast = AST::default();
        let a_char = ast.add(ASTNode::Character('a'));
        let b_char = ast.add(ASTNode::Character('b'));
        let union_node = ast.add(ASTNode::Union {
            left: a_char,
            right: b_char,
        });
        let star_node = ast.add(ASTNode::Star { child: union_node });
        let a2_char = ast.add(ASTNode::Character('a'));
        let cat1_node = ast.add(ASTNode::Concat {
            left: star_node,
            right: a2_char,
        });
        let b2_char = ast.add(ASTNode::Character('b'));
        let cat2_node = ast.add(ASTNode::Concat {
            left: cat1_node,
            right: b2_char,
        });
        let b3_char = ast.add(ASTNode::Character('b'));
        let cat3_node = ast.add(ASTNode::Concat {
            left: cat2_node,
            right: b3_char,
        });
        let end_char = ast.add(ASTNode::Character('#'));
        let root = ast.add(ASTNode::Concat {
            left: cat3_node,
            right: end_char,
        });

        let node_input_symbols: HashMap<NodeRef, HashSet<char>> = HashMap::from([
            (a_char, HashSet::from(['a'])),
            (b_char, HashSet::from(['b'])),
            (a2_char, HashSet::from(['a'])),
            (b2_char, HashSet::from(['b'])),
            (b3_char, HashSet::from(['b'])),
            (end_char, HashSet::from(['#'])),
        ]);

        let meta = AST::get_meta(&ast, root);
        let meta_pool = meta.get_pool();
        let root_meta = &meta_pool[root.0 as usize];
        assert_eq!(meta.size(), ast.size());
        assert_eq!(root_meta.nullable, false);
        assert_eq!(
            root_meta.first_pos,
            BTreeSet::from([a_char, b_char, a2_char])
        );
        assert_eq!(root_meta.last_pos, BTreeSet::from([end_char]));

        let followpos = get_follow_pos(&ast, &meta, root);
        // should be one followpos entry for each leaf node (character)
        assert_eq!(followpos.len(), 6);
        for nref in [a_char, a2_char, b_char, b2_char, b3_char] {
            assert!(followpos.contains_key(&nref));
        }

        // Assert values in followpos table
        assert_eq!(
            followpos.get(&a_char).unwrap(),
            &BTreeSet::from([a_char, b_char, a2_char])
        );
        assert_eq!(
            followpos.get(&b_char).unwrap(),
            &BTreeSet::from([a_char, b_char, a2_char])
        );
        assert_eq!(followpos.get(&a2_char).unwrap(), &BTreeSet::from([b2_char]));
        assert_eq!(followpos.get(&b2_char).unwrap(), &BTreeSet::from([b3_char]));
        assert_eq!(
            followpos.get(&b3_char).unwrap(),
            &BTreeSet::from([end_char])
        );

        let dfa = get_dfa(&ast, &meta, &followpos, root, &node_input_symbols);
        for entry in dfa.state_table {
            println!("{:?}", entry.0);
            for inner_entry in entry.1 {
                println!("  {:?}", inner_entry);
            }
        }
    }
}
