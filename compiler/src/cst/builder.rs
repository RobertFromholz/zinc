use crate::cst::{Cst, GreenNode, GreenToken, GreenTree, TokenKind, TreeKind};

impl Cst {
    pub fn tree<F>(mut self, kind: TreeKind, builder: F) -> Self
    where
        F: for <'cst> FnOnce(TreeBuilder<'cst>) -> TreeBuilder<'cst>,
    {
        build_tree(&mut self, kind, builder);
        self
    }
}

pub struct TreeBuilder<'cst> {
    cst: &'cst mut Cst,
}

impl<'cst, 'text> TreeBuilder<'cst> {
    pub fn tree<F>(self, kind: TreeKind, builder: F) -> Self
    where
        F: for <'child> FnOnce(TreeBuilder<'child>) -> TreeBuilder<'child>,
    {
        build_tree(self.cst, kind, builder);
        self
    }

    pub fn token(self, kind: TokenKind, length: usize) -> Self {
        self.cst.nodes.push(GreenNode::Token(GreenToken {
            kind,
            length,
        }));
        self
    }
}

fn build_tree<F>(cst: &mut Cst, kind: TreeKind, builder: F)
where
    F: for <'cst> FnOnce(TreeBuilder<'cst>) -> TreeBuilder<'cst>,
{
    // Find the index of the tree we will create.
    let index = cst.nodes.len();
    cst.nodes.push(GreenNode::Tree(GreenTree {
        kind,
        children: 0,
    }));
    let _ = builder(TreeBuilder {
        cst,
    });
    // Correctly patch the number of children for the newly created tree.
    let children = cst.nodes.len() - index - 1;
    let GreenNode::Tree(tree) = cst.nodes.get_mut(index).unwrap() else {
        unreachable!();
    };
    tree.children = children;
}
