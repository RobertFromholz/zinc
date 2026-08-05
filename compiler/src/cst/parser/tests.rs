use crate::cst::parser::Parser;
use crate::cst::{Cst, TokenKind, TreeKind};

fn verify(text: &str, expected: Cst) {
    let mut parser = Parser::new(text);
    parser.file();
    let actual = parser.finish();
    assert_eq!(actual, expected);
}

#[test]
fn parse_struct_declaration() {
    let text = r"
        struct Foo {}
    ";
    verify(text, Cst::empty(text)
        .tree(TreeKind::File, |builder| builder
            .token(TokenKind::Whitespace, 9)
            .tree(TreeKind::Items, |builder| builder
                .tree(TreeKind::Struct, |builder| builder
                    .token(TokenKind::Struct, 6)
                    .token(TokenKind::Whitespace, 1)
                    .token(TokenKind::Identifier, 3)
                    .token(TokenKind::Whitespace, 1)
                    .token(TokenKind::LeftBrace, 1)
                    .token(TokenKind::RightBrace, 1)
                )
                .token(TokenKind::Whitespace, 5),
            )))
}

#[test]
fn parse_struct_with_fields() {
    let text = r"
        struct Foo {
            foo: i32,
            bar: Foo
        }
    ";
    verify(text, Cst::empty(text)
        .tree(TreeKind::File, |builder| builder
            .token(TokenKind::Whitespace, 9)
            .tree(TreeKind::Items, |builder| builder
                .tree(TreeKind::Struct, |builder| builder
                    .token(TokenKind::Struct, 6)
                    .token(TokenKind::Whitespace, 1)
                    .token(TokenKind::Identifier, 3)
                    .token(TokenKind::Whitespace, 1)
                    .token(TokenKind::LeftBrace, 1)
                    .token(TokenKind::Whitespace, 13)
                    .tree(TreeKind::Field, |builder| builder
                        .token(TokenKind::Identifier, 3)
                        .token(TokenKind::Colon, 1)
                        .token(TokenKind::Whitespace, 1)
                        .tree(TreeKind::Type, |builder| builder
                            .token(TokenKind::Identifier, 3)
                        )
                    )
                    .token(TokenKind::Comma, 1)
                    .token(TokenKind::Whitespace, 13)
                    .tree(TreeKind::Field, |builder| builder
                        .token(TokenKind::Identifier, 3)
                        .token(TokenKind::Colon, 1)
                        .token(TokenKind::Whitespace, 1)
                        .tree(TreeKind::Type, |builder| builder
                            .token(TokenKind::Identifier, 3)
                        )
                    )
                    .token(TokenKind::Whitespace, 9)
                    .token(TokenKind::RightBrace, 1)
                )
                .token(TokenKind::Whitespace, 5)
            )));
}

#[test]
fn parse_function_declaration() {
    let text = r"
        fn foo_bar() -> Foo {}
    ";
    verify(text, Cst::empty(text)
        .tree(TreeKind::File, |builder| builder
            .token(TokenKind::Whitespace, 9)
            .tree(TreeKind::Items, |builder| builder
                .tree(TreeKind::Function, |builder| builder
                    .token(TokenKind::Fn, 2)
                    .token(TokenKind::Whitespace, 1)
                    .token(TokenKind::Identifier, 7)
                    .tree(TreeKind::Parameters, |builder| builder
                        .token(TokenKind::LeftParentheses, 1)
                        .token(TokenKind::RightParentheses, 1)
                    )
                    .token(TokenKind::Whitespace, 1)
                    .token(TokenKind::RightArrow, 2)
                    .token(TokenKind::Whitespace, 1)
                    .tree(TreeKind::Type, |builder| builder
                        .token(TokenKind::Identifier, 3)
                    )
                    .token(TokenKind::Whitespace, 1)
                    .tree(TreeKind::BlockExpression, |builder| builder
                        .token(TokenKind::LeftBrace, 1)
                        .token(TokenKind::RightBrace, 1)
                    )
                )
                .token(TokenKind::Whitespace, 5)
            )));
}
