//! Lexer responsible for converting source code into a stream of tokens.

mod cursor;

pub use cursor::Lexeme;

use super::{Span, Token, TokenKind};
use cursor::Cursor;
use std::collections::VecDeque;

/// A lexer to convert source code into a stream of tokens.
///
/// The lexer will not return combined tokens. A combined token (e.g. '->') is built up of other
/// tokens ('-' and '>'). The lexer is not aware whether a combined token is expected.
pub struct Lexer<'text> {
    text: &'text str,
    cursor: Cursor<'text>,
    queue: VecDeque<Token>,
}

impl<'text> Lexer<'text> {
    pub fn new(text: &'text str) -> Self {
        Self {
            text,
            cursor: Cursor::new(text),
            queue: VecDeque::new(),
        }
    }

    /// Consumes and returns the next token.
    pub fn next(&mut self) -> Option<Token> {
        self.queue.pop_front()
            .or_else(|| self.create())
    }

    /// Returns the next token without consuming it.
    pub fn peek(&mut self) -> Option<Token> {
        self.peek_at_offset(0)
    }

    /// Returns the token at the given offset without consuming it.
    pub fn peek_at_offset(&mut self, offset: usize) -> Option<Token> {
        while self.queue.len() <= offset {
            let next = self.create()?;
            self.queue.push_back(next);
        }
        let token = self.queue[offset].clone();
        Some(token)
    }

    /// Check if upcoming tokens can be combined into a new token of the expected kind.
    /// If so, consumes upcoming tokens and returns a new token.
    pub fn next_kind(&mut self, kind: TokenKind) -> Option<Token> {
        let token = self.peek_kind(kind)?;
        for _ in kind.decompose() {
            self.next();
        }
        Some(token)
    }

    /// Check if upcoming tokens can be combined into a new token of the expected kind.
    /// If so, returns a new token.
    pub fn peek_kind(&mut self, kind: TokenKind) -> Option<Token> {
        self.peek_kind_at_offset(kind, 0)
    }

    /// Check if upcoming tokens starting at the given offset from the current position can
    /// be combined into a new token of the expected kind.
    /// If so, returns a new token.
    pub fn peek_kind_at_offset(&mut self, kind: TokenKind, offset: usize) -> Option<Token> {
        kind.decompose().into_iter()
            .enumerate()
            .map(|(i, _)| self.peek_at_offset(i + offset))
            .collect::<Option<Vec<_>>>()
            .and_then(|parts| kind.combine(self.text, &parts))
    }

    fn create(&mut self) -> Option<Token> {
        let next = self.cursor.consume()?;
        let kind = match next {
            next if is_whitespace(next) => self.whitespace(),
            next if is_identifier_start(next) => self.identifier(),
            next if is_integer(next) => self.integer(),
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon,
            '=' => TokenKind::Equals,
            '-' => TokenKind::Minus,
            '>' => TokenKind::GreaterThan,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '(' => TokenKind::LeftParentheses,
            ')' => TokenKind::RightParentheses,
            _ => TokenKind::Unknown
        };
        let lexeme = self.cursor.close();
        Some(Token {
            kind,
            span: Span::new(self.text, lexeme.start_offset()..lexeme.end_offset())
        })
    }

    fn whitespace(&mut self) -> TokenKind {
        self.cursor.consume_while(is_whitespace);
        TokenKind::Whitespace
    }

    fn identifier(&mut self) -> TokenKind {
        self.cursor.consume_while(is_identifier_continue);
        let lexeme = self.cursor.current();
        let text = &self.text[lexeme.start_offset()..lexeme.end_offset()];
        TokenKind::try_from(text)
            .unwrap_or(TokenKind::Identifier)
    }

    fn integer(&mut self) -> TokenKind {
        self.cursor.consume_while(is_integer);
        TokenKind::Integer
    }
}

impl<'text> Iterator for Lexer<'text> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

fn is_whitespace(next: char) -> bool {
    next.is_whitespace()
}

fn is_identifier_start(next: char) -> bool {
    next == '_' || next.is_ascii_alphabetic()
}

fn is_identifier_continue(next: char) -> bool {
    is_identifier_start(next) || next.is_ascii_digit()
}

fn is_integer(next: char) -> bool {
    next.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next() {
        let text = "foo 123";
        let mut lexer = Lexer::new(text);
        assert_eq!(Some(Token::new(text, 0..3, TokenKind::Identifier)), lexer.next());
        assert_eq!(Some(Token::new(text, 3..4, TokenKind::Whitespace)), lexer.next());
        assert_eq!(Some(Token::new(text, 4..7, TokenKind::Integer)), lexer.next());
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_peek() {
        let text = "foo bar";
        let mut lexer = Lexer::new(text);
        assert_eq!(Some(Token::new(text, 0..3, TokenKind::Identifier)), lexer.peek());
        assert_eq!(Some(Token::new(text, 0..3, TokenKind::Identifier)), lexer.peek());
        assert_eq!(Some(Token::new(text, 0..3, TokenKind::Identifier)), lexer.next());
        assert_eq!(Some(Token::new(text, 3..4, TokenKind::Whitespace)), lexer.peek());
    }

    #[test]
    fn test_peek_at_offset() {
        let text = "foo bar";
        let mut lexer = Lexer::new(text);
        assert_eq!(Some(Token::new(text, 0..3, TokenKind::Identifier)), lexer.peek_at_offset(0));
        assert_eq!(Some(Token::new(text, 3..4, TokenKind::Whitespace)), lexer.peek_at_offset(1));
        assert_eq!(Some(Token::new(text, 4..7, TokenKind::Identifier)), lexer.peek_at_offset(2));
        assert_eq!(Some(Token::new(text, 0..3, TokenKind::Identifier)), lexer.next());
        assert_eq!(Some(Token::new(text, 3..4, TokenKind::Whitespace)), lexer.peek());
    }

    #[test]
    fn test_empty_input() {
        let text = "";
        let mut lexer = Lexer::new(text);
        assert_eq!(None, lexer.peek());
        assert_eq!(None, lexer.peek_at_offset(1));
        assert_eq!(None, lexer.next());
        assert_eq!(None, lexer.peek());
    }

    #[test]
    fn test_unknown() {
        let text = "§";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token::new(text, 0..2, TokenKind::Unknown)]
        );
    }

    #[test]
    fn test_emoji() {
        let text = "👨‍👩‍👧‍👦";
        let lexer = Lexer::new(text);
        // We currently don't handle multiple characters joined together.
        // As a result, we return all characters separately.
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token::new(text, 0..4, TokenKind::Unknown),
                Token::new(text, 4..7, TokenKind::Unknown),
                Token::new(text, 7..11, TokenKind::Unknown),
                Token::new(text, 11..14, TokenKind::Unknown),
                Token::new(text, 14..18, TokenKind::Unknown),
                Token::new(text, 18..21, TokenKind::Unknown),
                Token::new(text, 21..25, TokenKind::Unknown),
            ]
        );
    }

    #[test]
    fn test_identifier() {
        let text = "foo";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token::new(text, 0..3, TokenKind::Identifier)]
        );
    }

    #[test]
    fn test_identifier_with_number() {
        let text = "foo123";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token::new(text, 0..6, TokenKind::Identifier)]
        );
    }

    #[test]
    fn test_identifier_with_underscore() {
        let text = "foo_bar";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token::new(text, 0..7, TokenKind::Identifier)]
        );
    }

    #[test]
    fn test_identifier_starts_with_underscore() {
        let text = "_foo";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token::new(text, 0..4, TokenKind::Identifier)]
        );
    }

    #[test]
    fn test_whitespace() {
        let text = " \n\n \t ";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token::new(text, 0..text.len(), TokenKind::Whitespace)]
        );
    }

    #[test]
    fn test_integer() {
        let text = "123 456 0";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token::new(text, 0..3, TokenKind::Integer),
                Token::new(text, 3..4, TokenKind::Whitespace),
                Token::new(text, 4..7, TokenKind::Integer),
                Token::new(text, 7..8, TokenKind::Whitespace),
                Token::new(text, 8..9, TokenKind::Integer),
            ]
        );
    }

    #[test]
    fn test_keyword() {
        let text = "struct let fn";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token::new(text, 0..6, TokenKind::Struct),
                Token::new(text, 6..7, TokenKind::Whitespace),
                Token::new(text, 7..10, TokenKind::Let),
                Token::new(text, 10..11, TokenKind::Whitespace),
                Token::new(text, 11..13, TokenKind::Fn),
            ]
        );
    }

    #[test]
    fn test_punctuation() {
        let text = ",:;=-><";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token::new(text, 0..1, TokenKind::Comma),
                Token::new(text, 1..2, TokenKind::Colon),
                Token::new(text, 2..3, TokenKind::Semicolon),
                Token::new(text, 3..4, TokenKind::Equals),
                Token::new(text, 4..5, TokenKind::Minus),
                Token::new(text, 5..6, TokenKind::GreaterThan),
                Token::new(text, 6..7, TokenKind::Unknown),
            ]
        );
    }

    #[test]
    fn test_delimiter() {
        let text = "{}()";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token::new(text, 0..1, TokenKind::LeftBrace),
                Token::new(text, 1..2, TokenKind::RightBrace),
                Token::new(text, 2..3, TokenKind::LeftParentheses),
                Token::new(text, 3..4, TokenKind::RightParentheses),
            ]
        );
    }

    #[test]
    fn test_function() {
        let text = "fn foo(x: Bar) -> Bar { x }";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token::new(text, 0..2, TokenKind::Fn),
                Token::new(text, 2..3, TokenKind::Whitespace),
                Token::new(text, 3..6, TokenKind::Identifier),
                Token::new(text, 6..7, TokenKind::LeftParentheses),
                Token::new(text, 7..8, TokenKind::Identifier),
                Token::new(text, 8..9, TokenKind::Colon),
                Token::new(text, 9..10, TokenKind::Whitespace),
                Token::new(text, 10..13, TokenKind::Identifier),
                Token::new(text, 13..14, TokenKind::RightParentheses),
                Token::new(text, 14..15, TokenKind::Whitespace),
                Token::new(text, 15..16, TokenKind::Minus),
                Token::new(text, 16..17, TokenKind::GreaterThan),
                Token::new(text, 17..18, TokenKind::Whitespace),
                Token::new(text, 18..21, TokenKind::Identifier),
                Token::new(text, 21..22, TokenKind::Whitespace),
                Token::new(text, 22..23, TokenKind::LeftBrace),
                Token::new(text, 23..24, TokenKind::Whitespace),
                Token::new(text, 24..25, TokenKind::Identifier),
                Token::new(text, 25..26, TokenKind::Whitespace),
                Token::new(text, 26..27, TokenKind::RightBrace),
            ]
        );
    }

    #[test]
    fn test_struct() {
        let text = "struct Foo {}";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token::new(text, 0..6, TokenKind::Struct),
                Token::new(text, 6..7, TokenKind::Whitespace),
                Token::new(text, 7..10, TokenKind::Identifier),
                Token::new(text, 10..11, TokenKind::Whitespace),
                Token::new(text, 11..12, TokenKind::LeftBrace),
                Token::new(text, 12..13, TokenKind::RightBrace),
            ]
        )
    }

    #[test]
    fn test_next_kind_right_arrow() {
        let text = "->";
        let mut lexer = Lexer::new(text);
        let result = lexer.next_kind(TokenKind::RightArrow);
        assert_eq!(result, Some(Token::new(text, 0..2, TokenKind::RightArrow)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_next_kind_path_separator() {
        let text = "::";
        let mut lexer = Lexer::new(text);
        let result = lexer.next_kind(TokenKind::PathSeparator);
        assert_eq!(result, Some(Token::new(text, 0..2, TokenKind::PathSeparator)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_next_kind_fails_when_not_matching() {
        let text = "-;";
        let mut lexer = Lexer::new(text);
        let result = lexer.next_kind(TokenKind::RightArrow);
        assert_eq!(result, None);
        assert_eq!(lexer.next(), Some(Token::new(text, 0..1, TokenKind::Minus)));
    }

    #[test]
    fn test_peek_kind_right_arrow() {
        let text = "->";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind(TokenKind::RightArrow);
        assert_eq!(result, Some(Token::new(text, 0..2, TokenKind::RightArrow)));
        assert_eq!(lexer.peek(), Some(Token::new(text, 0..1, TokenKind::Minus)));
    }

    #[test]
    fn test_peek_kind_path_separator() {
        let text = "::";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind(TokenKind::PathSeparator);
        assert_eq!(result, Some(Token::new(text, 0..2, TokenKind::PathSeparator)));
        assert_eq!(lexer.peek(), Some(Token::new(text, 0..1, TokenKind::Colon)));
    }

    #[test]
    fn test_peek_kind_fails_when_not_matching() {
        let text = "-;";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind(TokenKind::RightArrow);
        assert_eq!(result, None);
    }

    #[test]
    fn test_peek_kind_at_offset_right_arrow() {
        let text = "foo ->";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind_at_offset(TokenKind::RightArrow, 2);
        assert_eq!(result, Some(Token::new(text, 4..6, TokenKind::RightArrow)));
        assert_eq!(lexer.peek(), Some(Token::new(text, 0..3, TokenKind::Identifier)));
    }

    #[test]
    fn test_peek_kind_at_offset_path_separator() {
        let text = "foo ::";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind_at_offset(TokenKind::PathSeparator, 2);
        assert_eq!(result, Some(Token::new(text, 4..6, TokenKind::PathSeparator)));
    }

    #[test]
    fn test_peek_kind_at_offset_zero() {
        let text = "->";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind_at_offset(TokenKind::RightArrow, 0);
        assert_eq!(result, Some(Token::new(text, 0..2, TokenKind::RightArrow)));
    }

    #[test]
    fn test_peek_kind_at_offset_fails_when_not_matching() {
        let text = "foo -;";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind_at_offset(TokenKind::RightArrow, 2);
        assert_eq!(result, None);
    }
}
