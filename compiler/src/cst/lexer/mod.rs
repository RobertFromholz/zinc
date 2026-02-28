//! Lexer responsible for converting source code into a stream of tokens.

mod lexeme;
mod token;

use std::collections::VecDeque;
pub use token::{Token, TokenKind};
use lexeme::{Lexeme, Cursor};
use crate::cst::lexer::token::KeywordKind;

/// A lexer to convert source code into a stream of tokens.
///
/// The lexer will not return combined tokens. A combined token (e.g. '->') is built up of other
/// tokens ('-' and '>'). The lexer is not aware whether a combined token is expected.
pub struct Lexer<'text> {
    cursor: Cursor<'text>,
    queue: VecDeque<Token<'text>>,
}

impl<'text> Lexer<'text> {
    pub fn new(text: &'text str) -> Self {
        Self {
            cursor: Cursor::new(text),
            queue: VecDeque::new(),
        }
    }

    /// Consumes and returns the next token.
    pub fn next(&mut self) -> Option<Token<'text>> {
        self.queue.pop_front()
            .or_else(|| self.create())
    }

    /// Returns the next token without consuming it.
    pub fn peek(&mut self) -> Option<Token<'text>> {
        self.peek_at_offset(0)
    }

    /// Returns the token at the given offset without consuming it.
    pub fn peek_at_offset(&mut self, offset: usize) -> Option<Token<'text>> {
        while self.queue.len() <= offset {
            let next = self.create()?;
            self.queue.push_back(next);
        }
        Some(self.queue[offset])
    }
    
    /// Check if upcoming tokens can be combined into a new token of the expected kind.
    /// If so, consumes upcoming tokens and returns a new token.
    pub fn next_kind(&mut self, kind: TokenKind) -> Option<Token<'text>> {
        todo!()
    }
    
    /// Check if upcoming tokens can be combined into a new token of the expected kind.
    /// If so, returns a new token.
    pub fn peek_kind(&mut self, kind: TokenKind) -> Option<Token<'text>> {
        self.peek_kind_at_offset(kind, 0)
    }
    
    /// Check if upcoming tokens starting at the given offset from the current position can
    /// be combined into a new token of the expected kind.
    /// If so, returns a new token.
    pub fn peek_kind_at_offset(&mut self, kind: TokenKind, offset: usize) -> Option<Token<'text>> {
        todo!()
    }

    fn create(&mut self) -> Option<Token<'text>> {
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
        let Lexeme { start_offset, text, length } = self.cursor.close();
        Some(Token {
            kind,
            start_offset,
            length,
            text,
        })
    }

    fn whitespace(&mut self) -> TokenKind {
        self.cursor.consume_while(is_whitespace);
        TokenKind::Whitespace
    }

    fn identifier(&mut self) -> TokenKind {
        self.cursor.consume_while(is_identifier_continue);
        let Lexeme { text, .. } = self.cursor.current();
        if let Ok(keyword) = KeywordKind::try_from(text) {
            TokenKind::Keyword(keyword)
        } else {
            TokenKind::Identifier
        }
    }

    fn integer(&mut self) -> TokenKind {
        self.cursor.consume_while(is_integer);
        TokenKind::Integer
    }
}

impl<'text> Iterator for Lexer<'text> {
    type Item = Token<'text>;

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
        let mut lexer = Lexer::new("foo 123");
        assert_eq!(Some(Token { kind: TokenKind::Identifier, start_offset: 0, length: 3, text: "foo" }), lexer.next());
        assert_eq!(Some(Token { kind: TokenKind::Whitespace, start_offset: 3, length: 1, text: " " }), lexer.next());
        assert_eq!(Some(Token { kind: TokenKind::Integer, start_offset: 4, length: 3, text: "123" }), lexer.next());
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_peek() {
        let mut lexer = Lexer::new("foo bar");
        assert_eq!(Some(Token { kind: TokenKind::Identifier, start_offset: 0, length: 3, text: "foo" }), lexer.peek());
        assert_eq!(Some(Token { kind: TokenKind::Identifier, start_offset: 0, length: 3, text: "foo" }), lexer.peek());
        assert_eq!(Some(Token { kind: TokenKind::Identifier, start_offset: 0, length: 3, text: "foo" }), lexer.next());
        assert_eq!(Some(Token { kind: TokenKind::Whitespace, start_offset: 3, length: 1, text: " " }), lexer.peek());
    }

    #[test]
    fn test_peek_at_offset() {
        let mut lexer = Lexer::new("foo bar");
        assert_eq!(Some(Token { kind: TokenKind::Identifier, start_offset: 0, length: 3, text: "foo" }), lexer.peek_at_offset(0));
        assert_eq!(Some(Token { kind: TokenKind::Whitespace, start_offset: 3, length: 1, text: " " }), lexer.peek_at_offset(1));
        assert_eq!(Some(Token { kind: TokenKind::Identifier, start_offset: 4, length: 3, text: "bar" }), lexer.peek_at_offset(2));
        assert_eq!(Some(Token { kind: TokenKind::Identifier, start_offset: 0, length: 3, text: "foo" }), lexer.next());
        assert_eq!(Some(Token { kind: TokenKind::Whitespace, start_offset: 3, length: 1, text: " " }), lexer.peek());
    }

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        assert_eq!(None, lexer.peek());
        assert_eq!(None, lexer.peek_at_offset(1));
        assert_eq!(None, lexer.next());
        assert_eq!(None, lexer.peek());
    }

    #[test]
    fn test_unknown() {
        let lexer = Lexer::new("§");
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token { kind: TokenKind::Unknown, start_offset: 0, length: 1, text: "§" }]
        );
    }

    #[test]
    fn test_emoji() {
        let lexer = Lexer::new("👨‍👩‍👧‍👦");
        // We currently don't handle multiple characters joined together.
        // As a result, we return all parts separately.
        // We also test that we correctly handle multibyte characters.
        // The start offset and length must be in characters and not in bytes.
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token { kind: TokenKind::Unknown, start_offset: 0, length: 1, text: "👨" },
                Token { kind: TokenKind::Unknown, start_offset: 1, length: 1, text: "‍" },
                Token { kind: TokenKind::Unknown, start_offset: 2, length: 1, text: "👩" },
                Token { kind: TokenKind::Unknown, start_offset: 3, length: 1, text: "‍" },
                Token { kind: TokenKind::Unknown, start_offset: 4, length: 1, text: "👧" },
                Token { kind: TokenKind::Unknown, start_offset: 5, length: 1, text: "‍" },
                Token { kind: TokenKind::Unknown, start_offset: 6, length: 1, text: "👦" },
            ]
        );
    }

    #[test]
    fn test_identifier() {
        let lexer = Lexer::new("foo");
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token { kind: TokenKind::Identifier, start_offset: 0, length: 3, text: "foo" }]
        );
    }

    #[test]
    fn test_identifier_with_number() {
        let lexer = Lexer::new("foo123");
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token { kind: TokenKind::Identifier, start_offset: 0, length: 6, text: "foo123" }]
        );
    }

    #[test]
    fn test_identifier_with_underscore() {
        let lexer = Lexer::new("foo_bar");
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token { kind: TokenKind::Identifier, start_offset: 0, length: 7, text: "foo_bar" }]
        );
    }

    #[test]
    fn test_identifier_starts_with_underscore() {
        let lexer = Lexer::new("_foo");
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token { kind: TokenKind::Identifier, start_offset: 0, length: 4, text: "_foo" }]
        );
    }

    #[test]
    fn test_whitespace() {
        let lexer = Lexer::new(" \n\n \t ");
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token { kind: TokenKind::Whitespace, start_offset: 0, length: 6, text: " \n\n \t " }]
        );
    }

    #[test]
    fn test_integer() {
        let lexer = Lexer::new("123 456 0");
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token { kind: TokenKind::Integer, start_offset: 0, length: 3, text: "123" },
                Token { kind: TokenKind::Whitespace, start_offset: 3, length: 1, text: " " },
                Token { kind: TokenKind::Integer, start_offset: 4, length: 3, text: "456" },
                Token { kind: TokenKind::Whitespace, start_offset: 7, length: 1, text: " " },
                Token { kind: TokenKind::Integer, start_offset: 8, length: 1, text: "0" },
            ]
        );
    }

    #[test]
    fn test_keyword() {
        let lexer = Lexer::new("module class let function constant mutable");
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token { kind: TokenKind::Keyword(KeywordKind::Module), start_offset: 0, length: 6, text: "module" },
                Token { kind: TokenKind::Whitespace, start_offset: 6, length: 1, text: " " },
                Token { kind: TokenKind::Keyword(KeywordKind::Class), start_offset: 7, length: 5, text: "class" },
                Token { kind: TokenKind::Whitespace, start_offset: 12, length: 1, text: " " },
                Token { kind: TokenKind::Keyword(KeywordKind::Field), start_offset: 13, length: 3, text: "let" },
                Token { kind: TokenKind::Whitespace, start_offset: 16, length: 1, text: " " },
                Token { kind: TokenKind::Keyword(KeywordKind::Function), start_offset: 17, length: 8, text: "function" },
                Token { kind: TokenKind::Whitespace, start_offset: 25, length: 1, text: " " },
                Token { kind: TokenKind::Keyword(KeywordKind::Constant), start_offset: 26, length: 8, text: "constant" },
                Token { kind: TokenKind::Whitespace, start_offset: 34, length: 1, text: " " },
                Token { kind: TokenKind::Keyword(KeywordKind::Mutable), start_offset: 35, length: 7, text: "mutable" },
            ]
        );
    }

    #[test]
    fn test_punctuation() {
        let lexer = Lexer::new(",:;=->");
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token { kind: TokenKind::Comma, start_offset: 0, length: 1, text: "," },
                Token { kind: TokenKind::Colon, start_offset: 1, length: 1, text: ":" },
                Token { kind: TokenKind::Semicolon, start_offset: 2, length: 1, text: ";" },
                Token { kind: TokenKind::Equals, start_offset: 3, length: 1, text: "=" },
                Token { kind: TokenKind::Minus, start_offset: 4, length: 1, text: "-" },
                Token { kind: TokenKind::GreaterThan, start_offset: 5, length: 1, text: ">" },
            ]
        );
    }

    #[test]
    fn test_delimiter() {
        let lexer = Lexer::new("{}()");
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token { kind: TokenKind::LeftBrace, start_offset: 0, length: 1, text: "{" },
                Token { kind: TokenKind::RightBrace, start_offset: 1, length: 1, text: "}" },
                Token { kind: TokenKind::LeftParentheses, start_offset: 2, length: 1, text: "(" },
                Token { kind: TokenKind::RightParentheses, start_offset: 3, length: 1, text: ")" },
            ]
        );
    }

    #[test]
    fn test_function() {
        let lexer = Lexer::new("function foo(x: Integer) -> Integer { x }");
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token { kind: TokenKind::Keyword(KeywordKind::Function), start_offset: 0, length: 8, text: "function" },
                Token { kind: TokenKind::Whitespace, start_offset: 8, length: 1, text: " " },
                Token { kind: TokenKind::Identifier, start_offset: 9, length: 3, text: "foo" },
                Token { kind: TokenKind::LeftParentheses, start_offset: 12, length: 1, text: "(" },
                Token { kind: TokenKind::Identifier, start_offset: 13, length: 1, text: "x" },
                Token { kind: TokenKind::Colon, start_offset: 14, length: 1, text: ":" },
                Token { kind: TokenKind::Whitespace, start_offset: 15, length: 1, text: " " },
                Token { kind: TokenKind::Identifier, start_offset: 16, length: 7, text: "Integer" },
                Token { kind: TokenKind::RightParentheses, start_offset: 23, length: 1, text: ")" },
                Token { kind: TokenKind::Whitespace, start_offset: 24, length: 1, text: " " },
                Token { kind: TokenKind::Minus, start_offset: 25, length: 1, text: "-" },
                Token { kind: TokenKind::GreaterThan, start_offset: 26, length: 1, text: ">" },
                Token { kind: TokenKind::Whitespace, start_offset: 27, length: 1, text: " " },
                Token { kind: TokenKind::Identifier, start_offset: 28, length: 7, text: "Integer" },
                Token { kind: TokenKind::Whitespace, start_offset: 35, length: 1, text: " " },
                Token { kind: TokenKind::LeftBrace, start_offset: 36, length: 1, text: "{" },
                Token { kind: TokenKind::Whitespace, start_offset: 37, length: 1, text: " " },
                Token { kind: TokenKind::Identifier, start_offset: 38, length: 1, text: "x" },
                Token { kind: TokenKind::Whitespace, start_offset: 39, length: 1, text: " " },
                Token { kind: TokenKind::RightBrace, start_offset: 40, length: 1, text: "}" },
            ]
        );
    }
}