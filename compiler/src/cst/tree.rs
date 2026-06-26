use crate::cst::token::Token;

/// A concrete syntax tree (CST).
///
/// A tree is a one-to-one representation of some object in the source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree<'text> {
    pub(super) kind: TreeKind,
    pub(super) children: Vec<Node<'text>>,
}

impl<'text> Tree<'text> {
    pub fn new(kind: TreeKind, children: impl Into<Vec<Node<'text>>>) -> Self {
        Self {
            kind,
            children: children.into(),
        }
    }
}

/// A node in the CST.
///
/// A node is either a leaf node (a token) or a composite node (a tree).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node<'text> {
    Tree(Tree<'text>),
    Token(Token<'text>),
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeKind {
    File,
    Module,
    Items,
    Struct,
    Function,
    Field,
    Initializer,
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
    Statement,
    Unknown,
}