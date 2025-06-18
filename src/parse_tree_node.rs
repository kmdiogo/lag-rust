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

    nullable: Option<bool>,
    first_pos: Option<HashSet<usize>>,
    last_pos: Option<HashSet<usize>>,
}

impl ParseTreeNode {
    pub fn new(
        node_type: ParseTreeNodeType,
        left: Box<Option<ParseTreeNode>>,
        right: Box<Option<ParseTreeNode>>,
        value: String,
    ) -> Self {
        Self {
            node_type,
            left,
            right,
            value,
            nullable: None,
            first_pos: None,
            last_pos: None,
        }
    }

    fn is_nullable(&mut self) -> bool {
        fn helper(node: Option<&mut ParseTreeNode>) -> bool {
            let n = match node {
                Some(n) => n,
                None => return true,
            };

            // Return from cache
            if let Some(nl) = n.nullable {
                return nl;
            }

            let result = match n.node_type {
                ParseTreeNodeType::Character | ParseTreeNodeType::Id | ParseTreeNodeType::Plus => {
                    false
                }
                ParseTreeNodeType::Question | ParseTreeNodeType::Star => true,
                ParseTreeNodeType::Union => {
                    helper((*n.left).as_mut()) || helper((*n.right).as_mut())
                }
                ParseTreeNodeType::Concat => {
                    helper((*n.left).as_mut()) && helper((*n.right).as_mut())
                }
            };
            n.nullable = Some(result);
            result
        }

        helper(Some(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_nullable() {
        // a*|b
        let a_node = ParseTreeNode {
            left: Box::new(None),
            right: Box::new(None),
            node_type: ParseTreeNodeType::Character,
            value: "a".to_string(),
            nullable: None,
            first_pos: None,
            last_pos: None,
        };
        let star_node = ParseTreeNode {
            left: Box::new(Some(a_node)),
            right: Box::new(None),
            node_type: ParseTreeNodeType::Star,
            value: "".to_string(),
            nullable: None,
            first_pos: None,
            last_pos: None,
        };
        let b_node = ParseTreeNode {
            left: Box::new(None),
            right: Box::new(None),
            node_type: ParseTreeNodeType::Character,
            value: "b".to_string(),
            nullable: None,
            first_pos: None,
            last_pos: None,
        };
        let mut union_node = ParseTreeNode {
            left: Box::new(Some(star_node)),
            right: Box::new(Some(b_node)),
            node_type: ParseTreeNodeType::Union,
            value: "".to_string(),
            nullable: None,
            first_pos: None,
            last_pos: None,
        };

        fn inorder(node: Option<&ParseTreeNode>, q: &mut Vec<Option<bool>>) {
            let n = match node {
                Some(n) => n,
                None => return,
            };
            inorder((*n.left).as_ref(), q);
            q.push(n.nullable);
            inorder((*n.right).as_ref(), q);
        };

        assert_eq!(union_node.is_nullable(), true);
        let mut inorder_nullable: Vec<Option<bool>> = Vec::new();
        inorder(Some(&union_node), &mut inorder_nullable);
        assert_eq!(
            inorder_nullable,
            vec![None, Some(true), Some(true), None] // NOTE: nullability of node 'b' will not be cached due to short-circuit evaluation for union nodes (|| operator)
        );
    }
}
