use crate::cst::parser::Parser;
use crate::cst::token::{Token, TokenKind};
use crate::cst::tree::{Node, Tree, TreeKind};

fn verify<'text>(text: &'text str, expected: impl Into<Vec<Node<'text>>>) {
    let mut parser = Parser::new(text);
    parser.file();
    let actual = parser.finish();
    let expected = Tree::new(TreeKind::File, expected);
    assert_eq!(expected, actual);
}

fn tree<'text>(kind: TreeKind, children: impl Into<Vec<Node<'text>>>) -> Node<'text> {
    Node::Tree(Tree::new(kind, children))
}

fn token(text: &'_ str, range: std::ops::Range<usize>, kind: TokenKind) -> Node<'_> {
    Node::Token(Token::new(text, range, kind))
}