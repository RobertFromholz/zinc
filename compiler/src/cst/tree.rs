use crate::cst::token::Token;
use crate::dot;
use std::fmt;
use std::fmt::Formatter;

/// A concrete syntax tree (CST).
///
/// A tree is a one-to-one representation of some object in the source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    pub(super) kind: TreeKind,
    // We need to store the start offset in the tree as well.
    // We can derive it from the tree's children, but only if it has any children.
    pub(super) start_offset: usize,
    pub(super) children: Vec<Node>,
}

impl Tree {
    pub fn new(kind: TreeKind, start_offset: usize, children: impl Into<Vec<Node>>) -> Self {
        Self {
            kind,
            start_offset,
            children: children.into(),
        }
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{} {{", self.kind)?;
            if self.children.is_empty() {
                write!(f, "}}")
            } else {
                writeln!(f, "")?;
                for child in &self.children {
                    let text = format!("{}", child)
                        .lines()
                        .map(|line| format!("\t{}", line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    writeln!(f, "{}", text)?;
                }
                writeln!(f, "}}")
            }
        } else {
            write!(f, "{}", self.kind)
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

impl fmt::Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Node::Tree(tree) => write!(f, "{}", tree),
            Node::Token(token) => write!(f, "{}", token),
            Node::Error(error) => write!(f, "{}", error),
        }
    }
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

impl fmt::Display for TreeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            TreeKind::File => "File",
            TreeKind::Module => "Module",
            TreeKind::Items => "Items",
            TreeKind::Struct => "Struct",
            TreeKind::Function => "Function",
            TreeKind::Field => "Field",
            TreeKind::Initializer => "Initializer",
            TreeKind::Parameters => "Parameters",
            TreeKind::Parameter => "Parameter",
            TreeKind::Type => "Type",
            TreeKind::Expression => "Expression",
            TreeKind::LiteralExpression => "Literal Expression",
            TreeKind::PrefixExpression => "prefix Expression",
            TreeKind::PathExpression => "Path Expression",
            TreeKind::CallExpression => "Call Expression",
            TreeKind::Arguments => "Arguments",
            TreeKind::ParenthesizedExpression => "Parenthesized Expression",
            TreeKind::BlockExpression => "Block Expression",
            TreeKind::Statement => "Statement",
            TreeKind::Unknown => "Unknown",
        })
    }
}

impl dot::Graph for Node {
    type Node<'a> = (String, &'a Node);
    type Edge<'a> = (&'a (String, &'a Node), (usize, &'a Node));

    fn nodes(&self) -> Vec<Self::Node<'_>> {
        vec![("0".to_owned(), self)]
    }

    fn edges(&self) -> Vec<Self::Edge<'_>> {
        vec![]
    }
}

impl<'a> dot::Node for (String, &'a Node) {
    type Edge<'b> = (&'b (String, &'b Node), (usize, &'b Node))
    where
        Self: 'b;

    fn id(&self) -> String {
        format!("{}", self.0)
    }

    fn label(&self) -> Option<String> {
        Some(format!("{}", self.1))
    }

    fn edges(&self) -> Vec<Self::Edge<'_>> {
        match self.1 {
            Node::Tree(tree) => tree.children.iter()
                .enumerate()
                .map(|(i, child)| (self, (i, child)))
                .collect(),
            _ => vec![]
        }
    }
}

impl<'a> dot::Edge for (&'a (String, &'a Node), (usize, &'a Node)) {
    type Node<'b> = (String, &'b Node)
    where
        Self: 'b;

    fn left_id(&self) -> String {
        format!("{}", self.0.0)
    }

    fn right_id(&self) -> String {
        format!("{}-{}", self.0.0, self.1.0)
    }

    fn right(&self) -> Option<Self::Node<'_>> {
        Some((format!("{}-{}", self.0.0, self.1.0), self.1.1))
    }
}