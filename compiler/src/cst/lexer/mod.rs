//! Lexer responsible for converting source code into a stream of tokens.

mod lexeme;

use std::collections::VecDeque;
use super::{Token, TokenKind, KeywordKind};
use lexeme::Cursor;

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
        let token = self.peek_kind(kind)?;
        for _ in kind.decompose() {
            self.next();
        }
        Some(token)
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
        kind.decompose().into_iter()
            .enumerate()
            .map(|(i, _)| self.peek_at_offset(i + offset))
            .collect::<Option<Vec<_>>>()
            .and_then(|parts| kind.combine(&parts))
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
        let span = self.cursor.close();
        Some(Token {
            kind,
            span,
        })
    }

    fn whitespace(&mut self) -> TokenKind {
        self.cursor.consume_while(is_whitespace);
        TokenKind::Whitespace
    }

    fn identifier(&mut self) -> TokenKind {
        self.cursor.consume_while(is_identifier_continue);
        let span = self.cursor.current();
        if let Ok(keyword) = KeywordKind::try_from(span.text()) {
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
    use crate::cst::Span;

    #[test]
    fn test_next() {
        let text = "foo 123";
        let mut lexer = Lexer::new(text);
        assert_eq!(Some(Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 0, length: "foo".len() } }), lexer.next());
        assert_eq!(Some(Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 3, length: " ".len() } }), lexer.next());
        assert_eq!(Some(Token { kind: TokenKind::Integer, span: Span { text, start_offset: 4, length: "123".len() } }), lexer.next());
        assert_eq!(None, lexer.next());
    }

    #[test]
    fn test_peek() {
        let text = "foo bar";
        let mut lexer = Lexer::new(text);
        assert_eq!(Some(Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 0, length: "foo".len() } }), lexer.peek());
        assert_eq!(Some(Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 0, length: "foo".len() } }), lexer.peek());
        assert_eq!(Some(Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 0, length: "foo".len() } }), lexer.next());
        assert_eq!(Some(Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 3, length: " ".len() } }), lexer.peek());
    }

    #[test]
    fn test_peek_at_offset() {
        let text = "foo bar";
        let mut lexer = Lexer::new(text);
        assert_eq!(Some(Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 0, length: "foo".len() } }), lexer.peek_at_offset(0));
        assert_eq!(Some(Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 3, length: " ".len() } }), lexer.peek_at_offset(1));
        assert_eq!(Some(Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 4, length: "bar".len() } }), lexer.peek_at_offset(2));
        assert_eq!(Some(Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 0, length: "foo".len() } }), lexer.next());
        assert_eq!(Some(Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 3, length: " ".len() } }), lexer.peek());
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
            vec![Token { kind: TokenKind::Unknown, span: Span { text, start_offset: 0, length: "§".len() } }]
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
                Token { kind: TokenKind::Unknown, span: Span { text, start_offset: 0, length: "👨".len() } },
                Token { kind: TokenKind::Unknown, span: Span { text, start_offset: 4, length: "‍".len() } },
                Token { kind: TokenKind::Unknown, span: Span { text, start_offset: 7, length: "👩".len() } },
                Token { kind: TokenKind::Unknown, span: Span { text, start_offset: 11, length: "‍".len() } },
                Token { kind: TokenKind::Unknown, span: Span { text, start_offset: 14, length: "👧".len() } },
                Token { kind: TokenKind::Unknown, span: Span { text, start_offset: 18, length: "‍".len() } },
                Token { kind: TokenKind::Unknown, span: Span { text, start_offset: 21, length: "👦".len() } },
            ]
        );
    }

    #[test]
    fn test_identifier() {
        let text = "foo";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 0, length: "foo".len() } }]
        );
    }

    #[test]
    fn test_identifier_with_number() {
        let text = "foo123";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 0, length: "foo123".len() } }]
        );
    }

    #[test]
    fn test_identifier_with_underscore() {
        let text = "foo_bar";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 0, length: "foo_bar".len() } }]
        );
    }

    #[test]
    fn test_identifier_starts_with_underscore() {
        let text = "_foo";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 0, length: "_foo".len() } }]
        );
    }

    #[test]
    fn test_whitespace() {
        let text = " \n\n \t ";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 0, length: " \n\n \t ".len() } }]
        );
    }

    #[test]
    fn test_integer() {
        let text = "123 456 0";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token { kind: TokenKind::Integer, span: Span { text, start_offset: 0, length: "123".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 3, length: " ".len() } },
                Token { kind: TokenKind::Integer, span: Span { text, start_offset: 4, length: "456".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 7, length: " ".len() } },
                Token { kind: TokenKind::Integer, span: Span { text, start_offset: 8, length: "0".len() } },
            ]
        );
    }

    #[test]
    fn test_keyword() {
        let text = "module class let function constant mutable";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token { kind: TokenKind::Keyword(KeywordKind::Module), span: Span { text, start_offset: 0, length: "module".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 6, length: " ".len() } },
                Token { kind: TokenKind::Keyword(KeywordKind::Class), span: Span { text, start_offset: 7, length: "class".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 12, length: " ".len() } },
                Token { kind: TokenKind::Keyword(KeywordKind::Field), span: Span { text, start_offset: 13, length: "let".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 16, length: " ".len() } },
                Token { kind: TokenKind::Keyword(KeywordKind::Function), span: Span { text, start_offset: 17, length: "function".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 25, length: " ".len() } },
                Token { kind: TokenKind::Keyword(KeywordKind::Constant), span: Span { text, start_offset: 26, length: "constant".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 34, length: " ".len() } },
                Token { kind: TokenKind::Keyword(KeywordKind::Mutable), span: Span { text, start_offset: 35, length: "mutable".len() } },
            ]
        );
    }

    #[test]
    fn test_punctuation() {
        let text = ",:;=->";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token { kind: TokenKind::Comma, span: Span { text, start_offset: 0, length: ",".len() } },
                Token { kind: TokenKind::Colon, span: Span { text, start_offset: 1, length: ":".len() } },
                Token { kind: TokenKind::Semicolon, span: Span { text, start_offset: 2, length: ";".len() } },
                Token { kind: TokenKind::Equals, span: Span { text, start_offset: 3, length: "=".len() } },
                Token { kind: TokenKind::Minus, span: Span { text, start_offset: 4, length: "-".len() } },
                Token { kind: TokenKind::GreaterThan, span: Span { text, start_offset: 5, length: ">".len() } },
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
                Token { kind: TokenKind::LeftBrace, span: Span { text, start_offset: 0, length: "{".len() } },
                Token { kind: TokenKind::RightBrace, span: Span { text, start_offset: 1, length: "}".len() } },
                Token { kind: TokenKind::LeftParentheses, span: Span { text, start_offset: 2, length: "(".len() } },
                Token { kind: TokenKind::RightParentheses, span: Span { text, start_offset: 3, length: ")".len() } },
            ]
        );
    }

    #[test]
    fn test_function() {
        let text = "function foo(x: Integer) -> Integer { x }";
        let lexer = Lexer::new(text);
        assert_eq!(
            lexer.collect::<Vec<_>>(),
            vec![
                Token { kind: TokenKind::Keyword(KeywordKind::Function), span: Span { text, start_offset: 0, length: "function".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 8, length: " ".len() } },
                Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 9, length: "foo".len() } },
                Token { kind: TokenKind::LeftParentheses, span: Span { text, start_offset: 12, length: "(".len() } },
                Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 13, length: "x".len() } },
                Token { kind: TokenKind::Colon, span: Span { text, start_offset: 14, length: ":".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 15, length: " ".len() } },
                Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 16, length: "Integer".len() } },
                Token { kind: TokenKind::RightParentheses, span: Span { text, start_offset: 23, length: ")".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 24, length: " ".len() } },
                Token { kind: TokenKind::Minus, span: Span { text, start_offset: 25, length: "-".len() } },
                Token { kind: TokenKind::GreaterThan, span: Span { text, start_offset: 26, length: ">".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 27, length: " ".len() } },
                Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 28, length: "Integer".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 35, length: " ".len() } },
                Token { kind: TokenKind::LeftBrace, span: Span { text, start_offset: 36, length: "{".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 37, length: " ".len() } },
                Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 38, length: "x".len() } },
                Token { kind: TokenKind::Whitespace, span: Span { text, start_offset: 39, length: " ".len() } },
                Token { kind: TokenKind::RightBrace, span: Span { text, start_offset: 40, length: "}".len() } },
            ]
        );
    }

    #[test]
    fn test_next_kind_right_arrow() {
        let text = "->";
        let mut lexer = Lexer::new(text);
        let result = lexer.next_kind(TokenKind::RightArrow);
        assert_eq!(result, Some(Token { kind: TokenKind::RightArrow, span: Span { text, start_offset: 0, length: "->".len() } }));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_next_kind_path_separator() {
        let text = "::";
        let mut lexer = Lexer::new(text);
        let result = lexer.next_kind(TokenKind::PathSeparator);
        assert_eq!(result, Some(Token { kind: TokenKind::PathSeparator, span: Span { text, start_offset: 0, length: "::".len() } }));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_next_kind_fails_when_not_matching() {
        let text = "-;";
        let mut lexer = Lexer::new(text);
        let result = lexer.next_kind(TokenKind::RightArrow);
        assert_eq!(result, None);
        assert_eq!(lexer.next(), Some(Token { kind: TokenKind::Minus, span: Span { text, start_offset: 0, length: "-".len() } }));
    }

    #[test]
    fn test_peek_kind_right_arrow() {
        let text = "->";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind(TokenKind::RightArrow);
        assert_eq!(result, Some(Token { kind: TokenKind::RightArrow, span: Span { text, start_offset: 0, length: "->".len() } }));
        assert_eq!(lexer.peek(), Some(Token { kind: TokenKind::Minus, span: Span { text, start_offset: 0, length: "-".len() } }));
    }

    #[test]
    fn test_peek_kind_path_separator() {
        let text = "::";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind(TokenKind::PathSeparator);
        assert_eq!(result, Some(Token { kind: TokenKind::PathSeparator, span: Span { text, start_offset: 0, length: "::".len() } }));
        assert_eq!(lexer.peek(), Some(Token { kind: TokenKind::Colon, span: Span { text, start_offset: 0, length: ":".len() } }));
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
        assert_eq!(result, Some(Token { kind: TokenKind::RightArrow, span: Span { text, start_offset: 4, length: "->".len() } }));
        assert_eq!(lexer.peek(), Some(Token { kind: TokenKind::Identifier, span: Span { text, start_offset: 0, length: "foo".len() } }));
    }

    #[test]
    fn test_peek_kind_at_offset_path_separator() {
        let text = "foo ::";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind_at_offset(TokenKind::PathSeparator, 2);
        assert_eq!(result, Some(Token { kind: TokenKind::PathSeparator, span: Span { text, start_offset: 4, length: "::".len() } }));
    }

    #[test]
    fn test_peek_kind_at_offset_zero() {
        let text = "->";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind_at_offset(TokenKind::RightArrow, 0);
        assert_eq!(result, Some(Token { kind: TokenKind::RightArrow, span: Span { text, start_offset: 0, length: "->".len() } }));
    }

    #[test]
    fn test_peek_kind_at_offset_fails_when_not_matching() {
        let text = "foo -;";
        let mut lexer = Lexer::new(text);
        let result = lexer.peek_kind_at_offset(TokenKind::RightArrow, 2);
        assert_eq!(result, None);
    }
}
