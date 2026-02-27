//! A lexeme stores the position of a token in the source code.
//!
//! A cursor is used to iterate over some source code and generate a stream of lexemes.

use std::str::Chars;

/// A lexeme stores the position of a token.
#[derive(Debug, PartialEq, Eq)]
pub struct Lexeme {
    start_offset: usize,
    length: usize,
}

/// An iterator over the source code.
///
/// Reads characters from the source code.
/// Stores the start offset and the length for the next lexeme.
pub struct Cursor<'a> {
    text: Chars<'a>,
    start_offset: usize,
    length: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text: text.chars(),
            start_offset: 0,
            length: 0,
        }
    }

    /// Close the current lexeme.
    pub fn close(&mut self) -> Lexeme {
        let lexeme = Lexeme {
            start_offset: self.start_offset,
            length: self.length,
        };
        self.start_offset += self.length;
        self.length = 0;
        lexeme
    }

    /// Consume the next character into the current token.
    ///
    /// Returns the consumed character.
    pub fn consume(&mut self) -> Option<char> {
        let next = self.text.next();
        if next.is_some() {
            self.length += 1;
        }
        next
    }

    /// If the next character matches some predicate, consume it into the current token.
    ///
    /// Returns the number of consumed characters.
    pub fn consume_while(&mut self, predicate: impl Fn(char) -> bool) -> usize {
        let mut consumed = 0;
        while self.peek().is_some_and(|next| predicate(next)) {
            self.consume();
            consumed += 1;
        }
        consumed
    }

    /// Peek the next character without consuming it.
    pub fn peek(&mut self) -> Option<char> {
        self.text.clone().next()
    }

    /// Peek the character at the given offset without consuming it.
    pub fn peek_at_offset(&mut self, offset: usize) -> Option<char> {
        self.text.clone().nth(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_lexeme_without_consuming() {
        let mut cursor = Cursor::new("");
        assert_eq!(cursor.close(), Lexeme { start_offset: 0, length: 0 });
    }

    #[test]
    fn test_close_lexeme_after_consuming_empty_text() {
        let mut cursor = Cursor::new("");
        assert_eq!(cursor.consume(), None);
        assert_eq!(cursor.close(), Lexeme { start_offset: 0, length: 0 });
    }

    #[test]
    fn test_close_lexeme() {
        let mut cursor = Cursor::new("abc");
        assert_eq!(cursor.consume(), Some('a'));
        assert_eq!(cursor.close(), Lexeme { start_offset: 0, length: 1 });
    }

    #[test]
    fn test_consume_while() {
        let mut cursor = Cursor::new("aaabc");
        assert_eq!(cursor.consume_while(|next| next == 'a'), 3);
        assert_eq!(cursor.close(), Lexeme { start_offset: 0, length: 3 });
    }

    #[test]
    fn test_peek() {
        let mut cursor = Cursor::new("abc");
        assert_eq!(cursor.peek(), Some('a'));
        assert_eq!(cursor.peek(), Some('a'));
        assert_eq!(cursor.consume(), Some('a'));
        assert_eq!(cursor.peek(), Some('b'));
    }

    #[test]
    fn test_peek_at_offset() {
        let mut cursor = Cursor::new("abc");
        assert_eq!(cursor.peek_at_offset(0), Some('a'));
        assert_eq!(cursor.peek_at_offset(1), Some('b'));
        assert_eq!(cursor.consume(), Some('a'));
        assert_eq!(cursor.peek_at_offset(0), Some('b'));
        assert_eq!(cursor.peek_at_offset(1), Some('c'));
    }
}