//! Lexer responsible for converting source code into a stream of tokens.

mod cursor;

use super::{GreenToken, TokenKind};
use cursor::Cursor;
use std::collections::VecDeque;

/// A lexer to convert source code into a stream of tokens.
///
/// The lexer will not return combined tokens. A combined token (e.g. '->') is built up of other
/// tokens ('-' and '>'). The lexer is not aware whether a combined token is expected.
pub struct Lexer<'text> {
    text: &'text str,
    cursor: Cursor<'text>,
    queue: VecDeque<GreenToken>,
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
    pub fn next(&mut self) -> Option<GreenToken> {
        self.queue.pop_front()
            .or_else(|| self.create())
    }

    /// Returns the next token without consuming it.
    pub fn peek(&mut self) -> Option<GreenToken> {
        self.peek_at_offset(0)
    }

    /// Returns the token at the given offset without consuming it.
    pub fn peek_at_offset(&mut self, offset: usize) -> Option<GreenToken> {
        while self.queue.len() <= offset {
            let next = self.create()?;
            self.queue.push_back(next);
        }
        let token = self.queue[offset].clone();
        Some(token)
    }

    /// Check if upcoming tokens can be combined into a new token of the expected kind.
    /// If so, consumes upcoming tokens and returns a new token.
    pub fn next_kind(&mut self, kind: TokenKind) -> Option<GreenToken> {
        let token = self.peek_kind(kind)?;
        for _ in kind.decompose() {
            self.next();
        }
        Some(token)
    }

    /// Check if upcoming tokens can be combined into a new token of the expected kind.
    /// If so, returns a new token.
    pub fn peek_kind(&mut self, kind: TokenKind) -> Option<GreenToken> {
        self.peek_kind_at_offset(kind, 0)
    }

    /// Check if upcoming tokens starting at the given offset from the current position can
    /// be combined into a new token of the expected kind.
    /// If so, returns a new token.
    pub fn peek_kind_at_offset(&mut self, kind: TokenKind, offset: usize) -> Option<GreenToken> {
        kind.decompose().into_iter()
            .enumerate()
            .map(|(i, _)| self.peek_at_offset(i + offset))
            .collect::<Option<Vec<_>>>()
            .and_then(|parts| kind.combine(&parts))
    }

    fn create(&mut self) -> Option<GreenToken> {
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
        Some(GreenToken { kind, length: lexeme.length })
    }

    fn whitespace(&mut self) -> TokenKind {
        self.cursor.consume_while(is_whitespace);
        TokenKind::Whitespace
    }

    fn identifier(&mut self) -> TokenKind {
        self.cursor.consume_while(is_identifier_continue);
        let lexeme = self.cursor.current();
        let text = &self.text[lexeme.start_offset..lexeme.start_offset + lexeme.length];
        TokenKind::try_from(text)
            .unwrap_or(TokenKind::Identifier)
    }

    fn integer(&mut self) -> TokenKind {
        self.cursor.consume_while(is_integer);
        TokenKind::Integer
    }
}

impl<'text> Iterator for Lexer<'text> {
    type Item = GreenToken;

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
        assert_eq!(Some(GreenToken { kind: TokenKind::Identifier, length: 3 }), lexer.next());
        assert_eq!(Some(GreenToken { kind: TokenKind::Whitespace, length: 1 }), lexer.next());
        assert_eq!(Some(GreenToken { kind: TokenKind::Integer, length: 3 }), lexer.next());
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_peek() {
        let text = "foo bar";
        let mut lexer = Lexer::new(text);
        assert_eq!(Some(GreenToken { kind: TokenKind::Identifier, length: 3 }), lexer.peek());
        assert_eq!(Some(GreenToken { kind: TokenKind::Identifier, length: 3 }), lexer.peek());
        assert_eq!(Some(GreenToken { kind: TokenKind::Identifier, length: 3 }), lexer.next());
        assert_eq!(Some(GreenToken { kind: TokenKind::Whitespace, length: 1 }), lexer.peek());
    }

    #[test]
    fn test_peek_at_offset() {
        let text = "foo bar";
        let mut lexer = Lexer::new(text);
        assert_eq!(Some(GreenToken { kind: TokenKind::Identifier, length: 3 }), lexer.peek_at_offset(0));
        assert_eq!(Some(GreenToken { kind: TokenKind::Whitespace, length: 1 }), lexer.peek_at_offset(1));
        assert_eq!(Some(GreenToken { kind: TokenKind::Identifier, length: 3 }), lexer.peek_at_offset(2));
        assert_eq!(Some(GreenToken { kind: TokenKind::Identifier, length: 3 }), lexer.next());
        assert_eq!(Some(GreenToken { kind: TokenKind::Whitespace, length: 1 }), lexer.peek());
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
            vec![GreenToken { kind: TokenKind::Unknown, length: 2 }]
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
                GreenToken { kind: TokenKind::Unknown, length: 4 },
                GreenToken { kind: TokenKind::Unknown, length: 3 },
                GreenToken { kind: TokenKind::Unknown, length: 4 },
                GreenToken { kind: TokenKind::Unknown, length: 3 },
                GreenToken { kind: TokenKind::Unknown, length: 4 },
                GreenToken { kind: TokenKind::Unknown, length: 3 },
                GreenToken { kind: TokenKind::Unknown, length: 4 },
            ]
        );
    }

    #[test]
    fn test_identifier() {
        let text = "foo";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![GreenToken { kind: TokenKind::Identifier, length: 3 }]
        );
    }

    #[test]
    fn test_identifier_with_number() {
        let text = "foo123";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![GreenToken { kind: TokenKind::Identifier, length: 6 }]
        );
    }

    #[test]
    fn test_identifier_with_underscore() {
        let text = "foo_bar";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![GreenToken { kind: TokenKind::Identifier, length: 7 }]
        );
    }

    #[test]
    fn test_identifier_starts_with_underscore() {
        let text = "_foo";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![GreenToken { kind: TokenKind::Identifier, length: 4 }]
        );
    }

    #[test]
    fn test_whitespace() {
        let text = " \n\n \t ";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![GreenToken { kind: TokenKind::Whitespace, length: text.len() }]
        );
    }

    #[test]
    fn test_integer() {
        let text = "123 456 0";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                GreenToken { kind: TokenKind::Integer, length: 3 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::Integer, length: 3 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::Integer, length: 1 },
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
                GreenToken { kind: TokenKind::Struct, length: 6 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::Let, length: 3 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::Fn, length: 2 },
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
                GreenToken { kind: TokenKind::Comma, length: 1 },
                GreenToken { kind: TokenKind::Colon, length: 1 },
                GreenToken { kind: TokenKind::Semicolon, length: 1 },
                GreenToken { kind: TokenKind::Equals, length: 1 },
                GreenToken { kind: TokenKind::Minus, length: 1 },
                GreenToken { kind: TokenKind::GreaterThan, length: 1 },
                GreenToken { kind: TokenKind::Unknown, length: 1 },
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
                GreenToken { kind: TokenKind::LeftBrace, length: 1 },
                GreenToken { kind: TokenKind::RightBrace, length: 1 },
                GreenToken { kind: TokenKind::LeftParentheses, length: 1 },
                GreenToken { kind: TokenKind::RightParentheses, length: 1 },
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
                GreenToken { kind: TokenKind::Fn, length: 2 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::Identifier, length: 3 },
                GreenToken { kind: TokenKind::LeftParentheses, length: 1 },
                GreenToken { kind: TokenKind::Identifier, length: 1 },
                GreenToken { kind: TokenKind::Colon, length: 1 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::Identifier, length: 3 },
                GreenToken { kind: TokenKind::RightParentheses, length: 1 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::Minus, length: 1 },
                GreenToken { kind: TokenKind::GreaterThan, length: 1 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::Identifier, length: 3 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::LeftBrace, length: 1 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::Identifier, length: 1 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::RightBrace, length: 1 },
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
                GreenToken { kind: TokenKind::Struct, length: 6 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::Identifier, length: 3 },
                GreenToken { kind: TokenKind::Whitespace, length: 1 },
                GreenToken { kind: TokenKind::LeftBrace, length: 1 },
                GreenToken { kind: TokenKind::RightBrace, length: 1 },
            ]
        )
    }

    #[test]
    fn test_next_kind_right_arrow() {
        let text = "->";
        let mut lexer = Lexer::new(text);
        let result = lexer.next_kind(TokenKind::RightArrow);
        assert_eq!(result, Some(GreenToken { kind: TokenKind::RightArrow, length: 2 }));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_next_kind_path_separator() {
        let text = "::";
        let mut lexer = Lexer::new(text);
        let result = lexer.next_kind(TokenKind::PathSeparator);
        assert_eq!(result, Some(GreenToken { kind: TokenKind::PathSeparator, length: 2 }));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_next_kind_fails_when_not_matching() {
        let text = "-;";
        let mut lexer = Lexer::new(text);
        let result = lexer.next_kind(TokenKind::RightArrow);
        assert_eq!(result, None);
        assert_eq!(lexer.next(), Some(GreenToken { kind: TokenKind::Minus, length: 1 }));
    }

    #[test]
    fn test_peek_kind_right_arrow() {
        let text = "->";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind(TokenKind::RightArrow);
        assert_eq!(result, Some(GreenToken { kind: TokenKind::RightArrow, length: 2 }));
        assert_eq!(lexer.peek(), Some(GreenToken { kind: TokenKind::Minus, length: 1 }));
    }

    #[test]
    fn test_peek_kind_path_separator() {
        let text = "::";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind(TokenKind::PathSeparator);
        assert_eq!(result, Some(GreenToken { kind: TokenKind::PathSeparator, length: 2 }));
        assert_eq!(lexer.peek(), Some(GreenToken { kind: TokenKind::Colon, length: 1 }));
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
        assert_eq!(result, Some(GreenToken { kind: TokenKind::RightArrow, length: 2 }));
        assert_eq!(lexer.peek(), Some(GreenToken { kind: TokenKind::Identifier, length: 3 }));
    }

    #[test]
    fn test_peek_kind_at_offset_path_separator() {
        let text = "foo ::";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind_at_offset(TokenKind::PathSeparator, 2);
        assert_eq!(result, Some(GreenToken { kind: TokenKind::PathSeparator, length: 2 }));
    }

    #[test]
    fn test_peek_kind_at_offset_zero() {
        let text = "->";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind_at_offset(TokenKind::RightArrow, 0);
        assert_eq!(result, Some(GreenToken { kind: TokenKind::RightArrow, length: 2 }));
    }

    #[test]
    fn test_peek_kind_at_offset_fails_when_not_matching() {
        let text = "foo -;";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind_at_offset(TokenKind::RightArrow, 2);
        assert_eq!(result, None);
    }
}
