//! The concrete syntax tree (CST) is a one-to-one representation of the source code.
//!
//! The parser registers symbols in the source code to a symbol table.

pub mod lexer;
pub mod parser;

#[cfg(test)]
pub mod tests;
pub mod builder;

use std::fmt;
use crate::dot;

/// The concrete syntax tree for a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cst {
    text: String,
    nodes: Vec<GreenNode>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GreenNode {
    Tree(GreenTree),
    Token(GreenToken),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GreenTree {
    pub kind: TreeKind,
    /// The number of descendants of this node, at any level.
    pub children: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GreenToken {
    pub kind: TokenKind,
    /// The length (in bytes) of this token.
    pub length: usize,
}

pub enum Node<'cst> {
    Tree(Tree<'cst>),
    Token(Token<'cst>),
}

pub struct Tree<'cst> {
    cst: &'cst Cst,
    index: usize,
    start_offset: usize,
    parent: Option<&'cst Tree<'cst>>,
}

pub struct Token<'cst> {
    cst: &'cst Cst,
    index: usize,
    start_offset: usize,
    parent: Option<&'cst Tree<'cst>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeKind {
    /// A file.
    File,

    /// A list of items: structures, functions, or fields.
    Items,

    /// A structure declaration.
    Struct,

    /// A function declaration.
    Function,

    /// A field declaration.
    Field,

    /// An initializer for a field. Wraps an expression.
    Initializer,

    /// A list of parameters in a function declaration.
    Parameters,

    /// A parameter in a function declaration.
    Parameter,

    /// A type reference.
    Type,

    /// An expression.
    Expression,

    /// A literal expression (i.e., a constant).
    LiteralExpression,

    /// A path to a declaration.
    PathExpression,

    /// A function call.
    CallExpression,

    /// A list of arguments to a function call.
    Arguments,

    /// A parenthesized expression.
    ParenthesizedExpression,

    /// A block expression.
    BlockExpression,

    /// A statement. Wraps an expression but doesn't return a value.
    Statement,

    /// An unknown node.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Any sequence of whitespace.
    Whitespace,

    /// An identifier.
    Identifier,

    /// An integer literal.
    Integer,

    /// `struct`
    Struct,

    /// `let`
    Let,

    /// `fn`
    Fn,

    /// `,`
    Comma,
    /// `;`
    Semicolon,
    /// `:`
    Colon,
    /// `=`
    Equals,

    // We technically don't use '-' token yet.
    // However, they are used to construct '->'.
    // The lexer shouldn't combine tokens since it doesn't know whether the syntax expects the
    // tokens individually or combined. As a result, it's easier to return a '-' token which can
    // be combined with the '>' token during parsing.

    /// `-`
    Minus,

    /// `>`
    GreaterThan,

    /// `->`
    RightArrow,

    /// `::`
    PathSeparator,

    /// `{`
    LeftBrace,

    /// `}`
    RightBrace,

    /// `(`
    LeftParentheses,

    /// `)`
    RightParentheses,

    /// Any unknown character.
    Unknown,
}

impl Cst {
    fn empty(text: impl Into<String>) -> Self {
        Cst {
            text: text.into(),
            nodes: vec![],
        }
    }
    
    /// Iterate over all roots in this tree.
    pub fn iter(&self) -> CstIter<'_> {
        CstIter {
            cst: self,
            parent: None,
            recursive: false,
            index: 0,
            start_offset: 0
        }
    }

    /// Iterate over all nodes in this tree.
    pub fn visit(&self) -> CstIter<'_> {
        CstIter {
            cst: self,
            parent: None,
            recursive: true,
            index: 0,
            start_offset: 0
        }
    }
}

impl<'cst> Node<'cst> {
    fn cst(&self) -> &Cst {
        match self {
            Node::Tree(tree) => tree.cst,
            Node::Token(token) => token.cst,
        }
    }

    fn index(&self) -> usize {
        match self {
            Node::Tree(tree) => tree.index,
            Node::Token(token) => token.index,
        }
    }

    fn green(&self) -> &GreenNode {
        self.cst().nodes.get(self.index()).unwrap()
    }
    
    pub fn parent(&self) -> Option<&'cst Tree<'cst>> {
        match self {
            Node::Tree(tree) => tree.parent(),
            Node::Token(token) => token.parent(),
        }
    }
    
    pub fn length(&self) -> usize {
        match self {
            Node::Tree(tree) => tree.length(),
            Node::Token(token) => token.length(),
        }
    }
    
    pub fn start_offset(&self) -> usize {
        match self {
            Node::Tree(tree) => tree.start_offset(),
            Node::Token(token) => token.start_offset(),
        }
    }
    
    pub fn end_offset(&self) -> usize {
        match self {
            Node::Tree(tree) => tree.end_offset(),
            Node::Token(token) => token.end_offset(),
        }
    }
    
    pub fn text(&self) -> &'cst str {
        match self {
            Node::Tree(tree) => tree.text(),
            Node::Token(token) => token.text(),
        }
    }
}

impl<'cst> Tree<'cst> {
    fn green(&self) -> &GreenTree {
        let node = self.cst.nodes.get(self.index).unwrap();
        match node {
            GreenNode::Tree(tree) => &tree,
            _ => panic!(),
        }
    }
    
    pub fn parent(&self) -> Option<&'cst Tree<'cst>> {
        self.parent
    }
    
    pub fn kind(&self) -> TreeKind {
        let green = self.green();
        green.kind
    }

    pub fn length(&self) -> usize {
        self.visit()
            .map(|node| match node {
                Node::Tree(_) => 0,
                Node::Token(token) => token.length()
            })
            .sum()
    }
    
    pub fn start_offset(&self) -> usize {
        self.start_offset
    }
    
    pub fn end_offset(&self) -> usize {
        self.start_offset() + self.length()
    }

    pub fn text(&self) -> &'cst str {
        &self.cst.text[self.start_offset()..self.end_offset()]
    }

    /// Iterate over all children of this node.
    pub fn iter(&self) -> CstIter<'_> {
        CstIter {
            cst: self.cst,
            parent: Some(self),
            recursive: false,
            index: self.index,
            start_offset: self.start_offset,
        }
    }

    /// Iterate over all descendants of this node.
    pub fn visit(&self) -> CstIter<'_> {
        CstIter {
            cst: self.cst,
            parent: Some(self),
            recursive: true,
            index: self.index,
            start_offset: self.start_offset,
        }
    }
}

impl<'cst> Token<'cst> {
    fn green(&self) -> &GreenToken {
        let node = self.cst.nodes.get(self.index).unwrap();
        match node {
            GreenNode::Token(token) => &token,
            _ => panic!(),
        }
    }
    
    pub fn parent(&self) -> Option<&'cst Tree<'cst>> {
        self.parent
    }
    
    pub fn kind(&self) -> TokenKind {
        let green = self.green();
        green.kind
    }

    pub fn length(&self) -> usize {
        let green = self.green();
        green.length
    }
    
    pub fn start_offset(&self) -> usize {
        self.start_offset
    }
    
    pub fn end_offset(&self) -> usize {
        self.start_offset() + self.length()
    }
    
    pub fn text(&self) -> &'cst str {
        &self.cst.text[self.start_offset()..self.end_offset()]
    }
}

impl<'cst> fmt::Display for Node<'cst> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Tree(tree) => fmt::Display::fmt(tree, f),
            Node::Token(token) => fmt::Display::fmt(token, f),
        }
    }
}

impl<'cst> fmt::Display for Tree<'cst> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            writeln!(f, "{} [{}-{}]", self.kind(), self.start_offset(), self.end_offset())?;
            for node in self.iter() {
                let text = format!("{}", node)
                    .lines()
                    .map(|line| format!("\t{}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                writeln!(f, "{}", text)?;
            }
            Ok(())
        } else {
            write!(f, "{} [{}-{}]", self.kind(), self.start_offset(), self.end_offset())
        }
    }
}

impl<'cst> fmt::Display for Token<'cst> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}-{}]", self.kind(), self.start_offset(), self.end_offset())
    }
}

/// An iterator over all children of a node.
pub struct CstIter<'cst> {
    cst: &'cst Cst,
    parent: Option<&'cst Tree<'cst>>,
    /// Whether to also descend into children's children.
    recursive: bool,
    index: usize,
    start_offset: usize,
}

impl<'cst> Iterator for CstIter<'cst> {
    type Item = Node<'cst>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(parent) = self.parent {
            let green = parent.green();
            if parent.index + green.children > self.index {
                // We have iterated over all our parent's children.
                return None;
            }
        }
        let next = self.cst.nodes.get(self.index)?;
        let node = match next {
            GreenNode::Tree(tree) => {
                if self.recursive {
                    self.index += 1
                } else {
                    self.index += tree.children;
                }
                let node = Tree {
                    cst: self.cst,
                    index: self.index,
                    start_offset: self.start_offset,
                    parent: self.parent,
                };
                Node::Tree(node)
            },
            GreenNode::Token(_) => {
                self.index += 1;
                let token = Token {
                    cst: self.cst,
                    index: self.index,
                    start_offset: self.start_offset,
                    parent: self.parent,
                };
                Node::Token(token)
            },
        };
        self.start_offset += node.length();
        Some(node)
    }
}

impl fmt::Display for TreeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            TreeKind::File => "File",
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

impl TokenKind {
    /// Try to combine a list of consecutive tokens into a new token of this type.
    fn combine(self, parts: &[GreenToken]) -> Option<GreenToken> {
        let expected = self.decompose().into_iter();
        let actual = parts.iter()
            .map(|token| token.kind);
        if expected.eq(actual) {
            let length = parts.iter()
                .map(|token| token.length)
                .sum();
            Some(GreenToken {
                kind: self,
                length,
            })
        } else {
            None
        }
    }

    /// Returns all parts that make up this token.
    fn decompose(self) -> Vec<TokenKind> {
        match self {
            TokenKind::RightArrow => vec![TokenKind::Minus, TokenKind::GreaterThan],
            TokenKind::PathSeparator => vec![TokenKind::Colon, TokenKind::Colon],
            _ => vec![self]
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            TokenKind::Whitespace => "whitespace",
            TokenKind::Identifier => "identifier",
            TokenKind::Integer => "integer",
            TokenKind::Struct => "struct",
            TokenKind::Let => "let",
            TokenKind::Fn => "fn",
            TokenKind::Comma => ",",
            TokenKind::Semicolon => ";",
            TokenKind::Colon => ":",
            TokenKind::Equals => "=",
            TokenKind::Minus => "-",
            TokenKind::GreaterThan => ">",
            TokenKind::RightArrow => "->",
            TokenKind::PathSeparator => "::",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::LeftParentheses => "(",
            TokenKind::RightParentheses => ")",
            TokenKind::Unknown => "unknown"
        })
    }
}

impl TryFrom<&str> for TokenKind {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "struct" => Ok(TokenKind::Struct),
            "let" => Ok(TokenKind::Let),
            "fn" => Ok(TokenKind::Fn),
            _ => Err(())
        }
    }
}

impl dot::Graph for Cst {
    type Node<'cst> = Node<'cst>
    where
        Self: 'cst;

    type Edge<'cst> = Node<'cst>
    where
        Self: 'cst;

    fn nodes(&self) -> Vec<Self::Node<'_>> {
        self.iter()
            .collect::<Vec<_>>()
    }

    fn edges(&self) -> Vec<Self::Edge<'_>> {
        vec![]
    }
}

impl<'cst> dot::Node for Node<'cst> {
    type Edge<'a> = Node<'a>
    where
        Self: 'a;

    fn id(&self) -> String {
        format!("{}", self.index())
    }

    fn label(&self) -> Option<String> {
        Some(format!("{}", self))
    }

    fn edges(&self) -> Vec<Self::Edge<'_>> {
        match self {
            Node::Tree(tree) => tree.iter()
                .collect::<Vec<_>>(),
            Node::Token(_) => vec![]
        }
    }
}

impl<'cst> dot::Node for &'cst Node<'cst> {
    type Edge<'a> = Node<'a>
    where
        Self: 'a;

    fn id(&self) -> String {
        format!("{}", self.index())
    }

    fn label(&self) -> Option<String> {
        Some(format!("{}", self))
    }

    fn edges(&self) -> Vec<Self::Edge<'_>> {
        match self {
            Node::Tree(tree) => tree.iter()
                .collect::<Vec<_>>(),
            Node::Token(_) => vec![]
        }
    }
}

impl<'cst> dot::Edge for Node<'cst> {
    type Node<'a> = &'a Node<'a>
    where
        Self: 'a;

    fn left_id(&self) -> String {
        let parent = self.parent().unwrap();
        format!("{}", parent.index)
    }

    fn right_id(&self) -> String {
        format!("{}", self.index())
    }

    fn right(&self) -> Option<Self::Node<'_>> {
        Some(self)
    }
}
