use crate::cst::parser::Parser;
use crate::cst::token::{Token, TokenKind};
use crate::cst::tree::{Node, Tree, TreeKind};

fn verify(text: &str, expected: impl Into<Vec<Node>>) {
    let mut parser = Parser::new(text);
    parser.file();
    let actual = parser.finish();
    let expected = Tree::new(TreeKind::File, 0, expected);
    assert_eq!(actual, expected);
}

fn tree<'text>(kind: TreeKind, start_offset: usize, children: impl Into<Vec<Node>>) -> Node {
    Node::Tree(Tree::new(kind, start_offset, children))
}

fn token(text: &'_ str, range: std::ops::Range<usize>, kind: TokenKind) -> Node {
    Node::Token(Token::new(text, range, kind))
}

#[test]
fn parse_class_declaration() {
    let text = r"
        struct Foo {}
    ";
    verify(text, &[
        token(text, 0..9, TokenKind::Whitespace),
        tree(TreeKind::Items, 9, &[
            tree(TreeKind::Struct, 9, &[
                token(text, 9..15, TokenKind::Struct),
                token(text, 15..16, TokenKind::Whitespace),
                token(text, 16..19, TokenKind::Identifier),
                token(text, 19..20, TokenKind::Whitespace),
                token(text, 20..21, TokenKind::LeftBrace),
                tree(TreeKind::Items, 21, &[]),
                token(text, 21..22, TokenKind::RightBrace),
            ]),
            token(text, 22..27, TokenKind::Whitespace),
        ])
    ]);
}

#[test]
fn parse_function_declaration() {
    let text = r"
        fn foo_bar() -> Foo {}
    ";
    verify(text, &[
        token(text, 0..9, TokenKind::Whitespace),
        tree(TreeKind::Items, 9, &[
            tree(TreeKind::Function, 9, &[
                token(text, 9..11, TokenKind::Fn),
                token(text, 11..12, TokenKind::Whitespace),
                token(text, 12..19, TokenKind::Identifier),
                tree(TreeKind::Parameters, 19,  &[
                    token(text, 19..20, TokenKind::LeftParentheses),
                    token(text, 20..21, TokenKind::RightParentheses),
                ]),
                token(text, 21..22, TokenKind::Whitespace),
                token(text, 22..24, TokenKind::RightArrow),
                token(text, 24..25, TokenKind::Whitespace),
                tree(TreeKind::Type, 25, &[
                    token(text, 25..28, TokenKind::Identifier),
                ]),
                token(text, 28..29, TokenKind::Whitespace),
                tree(TreeKind::BlockExpression, 29, &[
                    token(text, 29..30, TokenKind::LeftBrace),
                    token(text, 30..31, TokenKind::RightBrace),
                ])
            ]),
            token(text, 31..36, TokenKind::Whitespace)
        ]),
    ])
}
