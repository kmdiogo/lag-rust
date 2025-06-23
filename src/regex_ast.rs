use crate::arena::{Arena, ObjRef};
use log::debug;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt;

pub type NodeRef = ObjRef;

#[derive(Debug)]
pub enum ParseTreeNode {
    Character(char),
    Star { child: NodeRef },
    Plus { child: NodeRef },
    Question { child: NodeRef },
    Concat { left: NodeRef, right: NodeRef },
    Union { left: NodeRef, right: NodeRef },
}

impl fmt::Display for ParseTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseTreeNode::Character(char) => {
                write!(f, "Character({})", char)
            }
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
    pub nullable: bool,
    pub first_pos: BTreeSet<NodeRef>,
    pub last_pos: BTreeSet<NodeRef>,
}

pub type ParseTreeMeta = Arena<ParseTreeNodeMeta>;

impl ParseTree {
    pub fn get_meta(tree: &ParseTree, root: NodeRef) -> ParseTreeMeta {
        let mut meta: Vec<ParseTreeNodeMeta> = Vec::new();
        for _i in 0..tree.size() {
            meta.push(ParseTreeNodeMeta {
                nullable: false,
                first_pos: BTreeSet::new(),
                last_pos: BTreeSet::new(),
            })
        }

        fn calculate_meta(node_ref: NodeRef, meta: &mut Vec<ParseTreeNodeMeta>, ast: &ParseTree) {
            let ast_node = ast.get(node_ref);
            match ast_node {
                ParseTreeNode::Character { .. } => {
                    meta[node_ref.0 as usize] = ParseTreeNodeMeta {
                        last_pos: BTreeSet::from([node_ref]),
                        first_pos: BTreeSet::from([node_ref]),
                        nullable: false,
                    };
                }
                ParseTreeNode::Union { left, right } => {
                    calculate_meta(*left, meta, ast);
                    calculate_meta(*right, meta, ast);
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
                        ParseTreeNode::Star { child: _ } | ParseTreeNode::Question { child: _ } => {
                            true
                        }
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

    pub fn add_charset(tree: &mut ParseTree, char_set: &HashSet<char>) -> NodeRef {
        if char_set.is_empty() {
            panic!("empty char set provided. This indicates in upstream issue when setting a class identifier in the lookup table.");
        }

        if char_set.len() == 1 {
            tree.add(ParseTreeNode::Character(*char_set.iter().next().unwrap()));
        }

        let chars = Vec::from_iter(char_set.iter());
        let mut last_node: NodeRef = tree.add(ParseTreeNode::Character(*chars[0]));
        for char in chars.iter().skip(1) {
            let right = tree.add(ParseTreeNode::Character(**char));
            last_node = tree.add(ParseTreeNode::Union {
                left: last_node,
                right,
            });
        }
        last_node
    }
}

pub fn get_follow_pos(
    ast: &ParseTree,
    meta: &ParseTreeMeta,
    root: NodeRef,
) -> HashMap<NodeRef, BTreeSet<NodeRef>> {
    let end_node = match ast.get(root) {
        ParseTreeNode::Concat { left: _, right} => *right,
        _ => panic!("Root node is not a concatenation. Root of parse tree must be a cat node with right child of '#'"),
    };
    let mut follow_pos: HashMap<NodeRef, BTreeSet<NodeRef>> =
        HashMap::from([(end_node, BTreeSet::new())]);
    fn helper(
        ast: &ParseTree,
        meta: &ParseTreeMeta,
        node_ref: NodeRef,
        follow_pos: &mut HashMap<NodeRef, BTreeSet<NodeRef>>,
    ) {
        let node = meta.get(node_ref);
        // debug!(
        //     "Calculating followpos for {:?}({:?})",
        //     ast.get(node_ref),
        //     node_ref
        // );
        match ast.get(node_ref) {
            ParseTreeNode::Concat { left, right, .. } => {
                let follow_nodes = &meta.get(*right).first_pos;
                for nref in &meta.get(*left).last_pos {
                    // debug!(
                    //     "  followpos({:?}({:?})) = {:?}",
                    //     ast.get(*nref),
                    //     nref,
                    //     follow_nodes,
                    // );
                    let nref_followpos = follow_pos.entry(*nref).or_insert(BTreeSet::new());
                    nref_followpos.extend(follow_nodes.into_iter())
                }
                helper(ast, meta, *left, follow_pos);
                helper(ast, meta, *right, follow_pos);
            }
            ParseTreeNode::Star { child } | ParseTreeNode::Plus { child } => {
                let follow_nodes = &node.first_pos;
                for nref in &node.last_pos {
                    let nref_followpos = follow_pos.entry(*nref).or_insert(BTreeSet::new());
                    nref_followpos.extend(follow_nodes.into_iter())
                }
                helper(ast, meta, *child, follow_pos);
            }
            ParseTreeNode::Union { left, right } => {
                helper(ast, meta, *left, follow_pos);
                helper(ast, meta, *right, follow_pos);
            }
            _ => {}
        };
    }
    helper(ast, meta, root, &mut follow_pos);
    follow_pos
}

fn group_nodes_by_char(
    node_refs: &BTreeSet<NodeRef>,
    ast: &ParseTree,
) -> HashMap<char, BTreeSet<NodeRef>> {
    let mut groups: HashMap<char, BTreeSet<NodeRef>> = HashMap::new();
    for node_ref in node_refs {
        let char = match ast.get(*node_ref) {
            ParseTreeNode::Character(char) => char,
            other => {
                panic!("node {:?} is not a character node. This indicates an issue in upstream ast or ast meta code.", other)
            }
        };
        // println!("{:?} => {}", node_ref, char);
        let group = groups.entry(*char).or_insert(BTreeSet::new());
        group.insert(*node_ref);
    }
    groups
}

pub struct DFA {
    pub state_table: HashMap<BTreeSet<NodeRef>, HashMap<char, BTreeSet<NodeRef>>>,
}

pub fn get_dfa(
    ast: &ParseTree,
    meta: &ParseTreeMeta,
    followpos: &HashMap<NodeRef, BTreeSet<NodeRef>>,
    root: NodeRef,
) -> DFA {
    let root_meta = meta.get(root);
    let mut accepting_states = HashSet::new();
    let mut state_table: HashMap<BTreeSet<NodeRef>, HashMap<char, BTreeSet<NodeRef>>> =
        HashMap::from([(root_meta.first_pos.clone(), HashMap::new())]);
    let mut queue: VecDeque<BTreeSet<NodeRef>> = VecDeque::from([root_meta.first_pos.clone()]);
    while queue.len() > 0 {
        let cur_state = queue.pop_front().unwrap();
        debug!("Doing {:?}", cur_state);
        let grouped_nodes = group_nodes_by_char(&cur_state, ast);
        for (char, node_refs) in grouped_nodes {
            if char == '#' {
                accepting_states.insert(cur_state.clone());
                continue;
            }
            debug!("  {} groups => {:?}", char, node_refs);
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
            state_transition.insert(char, target_state.clone());
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
        let mut ast = ParseTree::default();
        let a_char = ast.add(ParseTreeNode::Character('a'));
        let b_char = ast.add(ParseTreeNode::Character('b'));
        let union_node = ast.add(ParseTreeNode::Union {
            left: a_char,
            right: b_char,
        });
        let star_node = ast.add(ParseTreeNode::Star { child: union_node });
        let a2_char = ast.add(ParseTreeNode::Character('a'));
        let cat1_node = ast.add(ParseTreeNode::Concat {
            left: star_node,
            right: a2_char,
        });
        let b2_char = ast.add(ParseTreeNode::Character('b'));
        let cat2_node = ast.add(ParseTreeNode::Concat {
            left: cat1_node,
            right: b2_char,
        });
        let b3_char = ast.add(ParseTreeNode::Character('b'));
        let cat3_node = ast.add(ParseTreeNode::Concat {
            left: cat2_node,
            right: b3_char,
        });
        let end_char = ast.add(ParseTreeNode::Character('#'));
        let root = ast.add(ParseTreeNode::Concat {
            left: cat3_node,
            right: end_char,
        });

        let meta = ParseTree::get_meta(&ast, root);
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

        let dfa = get_dfa(&ast, &meta, &followpos, root);
        for entry in dfa.state_table {
            println!("{:?}", entry.0);
            for inner_entry in entry.1 {
                println!("  {:?}", inner_entry);
            }
        }
    }
}
