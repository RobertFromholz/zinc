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

fn token(kind: TokenKind, length: usize) -> Node {
    Node::Token(Token::new(kind, length))
}

#[test]
fn parse_struct_declaration() {
    let text = r"
        struct Foo {}
    ";
    verify(text, &[
        token(TokenKind::Whitespace, 9),
        tree(TreeKind::Items, 9, &[
            tree(TreeKind::Struct, 9, &[
                token(TokenKind::Struct, 6),
                token(TokenKind::Whitespace, 1),
                token(TokenKind::Identifier, 3),
                token(TokenKind::Whitespace, 1),
                token(TokenKind::LeftBrace, 1),
                token(TokenKind::RightBrace, 1),
            ]),
            token(TokenKind::Whitespace, 5),
        ])
    ]);
}

#[test]
fn parse_struct_with_fields() {
    let text = r"
        struct Foo {
            foo: i32,
            bar: Foo
        }
    ";
    verify(text, &[
        token(TokenKind::Whitespace, 9),
        tree(TreeKind::Items, 9, &[
            tree(TreeKind::Struct, 9, &[
                token(TokenKind::Struct, 6),
                token(TokenKind::Whitespace, 1),
                token(TokenKind::Identifier, 3),
                token(TokenKind::Whitespace, 1),
                token(TokenKind::LeftBrace, 1),
                token(TokenKind::Whitespace, 13),
                tree(TreeKind::Field, 34, &[
                    token(TokenKind::Identifier, 3),
                    token(TokenKind::Colon, 1),
                    token(TokenKind::Whitespace, 1),
                    tree(TreeKind::Type, 39, &[
                        token(TokenKind::Identifier, 3),
                    ])
                ]),
                token(TokenKind::Comma, 1),
                token(TokenKind::Whitespace, 13),
                tree(TreeKind::Field, 56, &[
                    token(TokenKind::Identifier, 3),
                    token(TokenKind::Colon, 1),
                    token(TokenKind::Whitespace, 1),
                    tree(TreeKind::Type, 61, &[
                        token(TokenKind::Identifier, 3),
                    ])
                ]),
                token(TokenKind::Whitespace, 9),
                token(TokenKind::RightBrace, 1),
            ]),
            token(TokenKind::Whitespace, 5),
        ])
    ]);
}

#[test]
fn parse_function_declaration() {
    let text = r"
        fn foo_bar() -> Foo {}
    ";
    verify(text, &[
        token(TokenKind::Whitespace, 9),
        tree(TreeKind::Items, 9, &[
            tree(TreeKind::Function, 9, &[
                token(TokenKind::Fn, 2),
                token(TokenKind::Whitespace, 1),
                token(TokenKind::Identifier, 7),
                tree(TreeKind::Parameters, 19,  &[
                    token(TokenKind::LeftParentheses, 1),
                    token(TokenKind::RightParentheses, 1),
                ]),
                token(TokenKind::Whitespace, 1),
                token(TokenKind::RightArrow, 2),
                token(TokenKind::Whitespace, 1),
                tree(TreeKind::Type, 25, &[
                    token(TokenKind::Identifier, 3),
                ]),
                token(TokenKind::Whitespace, 1),
                tree(TreeKind::BlockExpression, 29, &[
                    token(TokenKind::LeftBrace, 1),
                    token(TokenKind::RightBrace, 1),
                ])
            ]),
            token(TokenKind::Whitespace, 5)
        ]),
    ])
}
