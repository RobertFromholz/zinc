use crate::cst::token::Token;

/// A concrete syntax tree (CST).
///
/// A tree is a one-to-one representation of some object in the source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    pub(super) kind: TreeKind,
    pub(super) children: Vec<Node>,
}

impl Tree {
    pub fn new(kind: TreeKind, children: impl Into<Vec<Node>>) -> Self {
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
pub enum Node {
    Tree(Tree),
    Token(Token),
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