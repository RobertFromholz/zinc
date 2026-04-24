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

#[test]
fn parse_module() {
    let text = r"
        module foo;
    ";
    verify(text, &[
        token(text, 0..9, TokenKind::Whitespace),
        tree(TreeKind::Module, &[
            token(text, 9..15, TokenKind::Module),
            token(text, 15..16, TokenKind::Whitespace),
            token(text, 16..19, TokenKind::Identifier),
            token(text, 19..20, TokenKind::Semicolon),
        ]),
        token(text, 20..25, TokenKind::Whitespace),
        tree(TreeKind::Items, &[])
    ]);
}