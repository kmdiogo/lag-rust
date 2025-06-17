use std::collections::HashSet;

#[derive(Debug)]
pub enum ParseTreeNodeType {
    Concat,
    Character,
    Id,
    Plus,
    Question,
    Star,
    Union,
}
pub struct ParseTreeNode {
    pub node_type: ParseTreeNodeType,
    pub left: Box<Option<ParseTreeNode>>,
    pub right: Box<Option<ParseTreeNode>>,
    pub value: String,

    nullabe: Option<bool>,
    first_pos: Option<HashSet<usize>>,
    last_pos: Option<HashSet<usize>>,
}

impl ParseTreeNode {
    fn is_nullable(&self) -> bool {
        fn helper(node: Option<&ParseTreeNode>) -> bool {
            let n = match node {
                Some(n) => n,
                None => return true,
            };

            // Return from cache
            if let Some(nl) = n.nullabe {
                return nl;
            }

            match n.node_type {
                ParseTreeNodeType::Character | ParseTreeNodeType::Id | ParseTreeNodeType::Plus => {
                    false
                }
                ParseTreeNodeType::Question | ParseTreeNodeType::Star => true,
                ParseTreeNodeType::Union => {
                    helper((*n.left).as_ref()) || helper((*n.right).as_ref())
                }
                ParseTreeNodeType::Concat => {
                    helper((*n.left).as_ref()) && helper((*n.right).as_ref())
                }
            }
        }

        helper(Some(self))
    }
}
