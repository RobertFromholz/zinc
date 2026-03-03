use crate::cst::token::Token;

/// A concrete syntax tree (CST).
///
/// A tree is a one-to-one representation of some object in the source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree<'text> {
    pub(super) kind: TreeKind,
    pub(super) children: Vec<Node<'text>>,
}

/// A node in a tree.
/// A node is either a leaf node (a token) or a composite node (a tree).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node<'text> {
    Tree(Tree<'text>),
    Token(Token<'text>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeKind {
    File,
    Module,
    Class,
    Function,
    Field,
    Inherits,
    Parameters,
    Parameter,
    Type,
    Expression,
    LiteralExpression,
    PrefixExpression,
    PathExpression,
    CallExpression,
    Arguments,
    ParenthesizedExpression,
    BlockExpression,
    Statement
}