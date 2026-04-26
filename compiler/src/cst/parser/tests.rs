use crate::cst::parser::Parser;
use crate::cst::token::{Token, TokenKind};
use crate::cst::tree::{Node, Tree, TreeKind};

fn verify<'text>(text: &'text str, expected: impl Into<Vec<Node<'text>>>) {
    let mut parser = Parser::new(text);
    parser.file();
    let actual = parser.finish();
    let expected = Tree::new(TreeKind::File, expected);
    assert_eq!(actual, expected);
}

fn tree<'text>(kind: TreeKind, children: impl Into<Vec<Node<'text>>>) -> Node<'text> {
    Node::Tree(Tree::new(kind, children))
}

fn token(text: &'_ str, range: std::ops::Range<usize>, kind: TokenKind) -> Node<'_> {
    Node::Token(Token::new(text, range, kind))
}

#[test]
fn parse_class_declaration() {
    let text = r"
        class Foo {}
    ";
    verify(text, &[
        token(text, 0..9, TokenKind::Whitespace),
        tree(TreeKind::Items, &[
            tree(TreeKind::Class, &[
                token(text, 9..14, TokenKind::Class),
                token(text, 14..15, TokenKind::Whitespace),
                token(text, 15..18, TokenKind::Identifier),
                token(text, 18..19, TokenKind::Whitespace),
                token(text, 19..20, TokenKind::LeftBrace),
                tree(TreeKind::Items, &[]),
                token(text, 20..21, TokenKind::RightBrace),
            ]),
            token(text, 21..26, TokenKind::Whitespace),
        ])
    ]);
}

#[test]
fn parse_function_declaration() {
    let text = r"
        function foo_bar() -> Foo {}
    ";
    verify(text, &[
        token(text, 0..9, TokenKind::Whitespace),
        tree(TreeKind::Items, &[
            tree(TreeKind::Function, &[
                token(text, 9..17, TokenKind::Function),
                token(text, 17..18, TokenKind::Whitespace),
                token(text, 18..25, TokenKind::Identifier),
                tree(TreeKind::Parameters, &[
                    token(text, 25..26, TokenKind::LeftParentheses),
                    token(text, 26..27, TokenKind::RightParentheses),
                ]),
                token(text, 27..28, TokenKind::Whitespace),
                token(text, 28..30, TokenKind::RightArrow),
                token(text, 30..31, TokenKind::Whitespace),
                tree(TreeKind::Type, &[
                    tree(TreeKind::PathExpression, &[
                        token(text, 31..34, TokenKind::Identifier),
                    ])
                ]),
                token(text, 34..35, TokenKind::Whitespace),
                tree(TreeKind::BlockExpression, &[
                    token(text, 35..36, TokenKind::LeftBrace),
                    token(text, 36..37, TokenKind::RightBrace),
                ])
            ]),
            token(text, 37..42, TokenKind::Whitespace)
        ]),
    ])
}
