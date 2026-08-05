use crate::cst::parser::Parser;
use crate::cst::token::{Token, TokenKind};
use crate::cst::tree::{Node, Tree, TreeKind};

fn verify(text: &str, expected: impl Into<Vec<Node>>) {
    let mut parser = Parser::new(text);
    parser.file();
    let actual = parser.finish();
    let expected = Tree::new(TreeKind::File, expected);
    assert_eq!(actual, expected);
}

fn tree<'text>(kind: TreeKind, children: impl Into<Vec<Node>>) -> Node {
    Node::Tree(Tree::new(kind, children))
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
        tree(TreeKind::Items, &[
            tree(TreeKind::Struct, &[
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
        tree(TreeKind::Items, &[
            tree(TreeKind::Struct, &[
                token(TokenKind::Struct, 6),
                token(TokenKind::Whitespace, 1),
                token(TokenKind::Identifier, 3),
                token(TokenKind::Whitespace, 1),
                token(TokenKind::LeftBrace, 1),
                token(TokenKind::Whitespace, 13),
                tree(TreeKind::Field, &[
                    token(TokenKind::Identifier, 3),
                    token(TokenKind::Colon, 1),
                    token(TokenKind::Whitespace, 1),
                    tree(TreeKind::Type, &[
                        token(TokenKind::Identifier, 3),
                    ])
                ]),
                token(TokenKind::Comma, 1),
                token(TokenKind::Whitespace, 13),
                tree(TreeKind::Field, &[
                    token(TokenKind::Identifier, 3),
                    token(TokenKind::Colon, 1),
                    token(TokenKind::Whitespace, 1),
                    tree(TreeKind::Type, &[
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
        tree(TreeKind::Items, &[
            tree(TreeKind::Function, &[
                token(TokenKind::Fn, 2),
                token(TokenKind::Whitespace, 1),
                token(TokenKind::Identifier, 7),
                tree(TreeKind::Parameters, &[
                    token(TokenKind::LeftParentheses, 1),
                    token(TokenKind::RightParentheses, 1),
                ]),
                token(TokenKind::Whitespace, 1),
                token(TokenKind::RightArrow, 2),
                token(TokenKind::Whitespace, 1),
                tree(TreeKind::Type, &[
                    token(TokenKind::Identifier, 3),
                ]),
                token(TokenKind::Whitespace, 1),
                tree(TreeKind::BlockExpression, &[
                    token(TokenKind::LeftBrace, 1),
                    token(TokenKind::RightBrace, 1),
                ])
            ]),
            token(TokenKind::Whitespace, 5)
        ]),
    ])
}
