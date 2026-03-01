//! A lexeme stores the position of a token in the source code.
//!
//! A cursor is used to iterate over some source code and generate a stream of lexemes.

use std::str::Chars;

/// A lexeme stores the position of a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lexeme<'text> {
    pub(super) start_offset: usize,
    pub(super) text: &'text str,
}

/// An iterator to convert source code into a stream of lexemes.
///
/// Consumes characters into a lexeme.
/// Does not know the type of lexeme being consumed.
pub struct Cursor<'text> {
    text: &'text str,
    iterator: Chars<'text>,
    start_offset: usize,
    length: usize,
}

impl<'text> Cursor<'text> {
    pub fn new(text: &'text str) -> Self {
        Self {
            text,
            iterator: text.chars(),
            start_offset: 0,
            length: 0,
        }
    }

    /// Returns the current lexeme.
    pub fn current(&self) -> Lexeme<'text> {
        Lexeme {
            start_offset: self.start_offset,
            text: &self.text[self.start_offset..self.start_offset + self.length],
        }
    }

    /// Close the current lexeme.
    pub fn close(&mut self) -> Lexeme<'text> {
        let current = self.current();
        self.start_offset += self.length;
        self.length = 0;
        current
    }

    /// Consume the next character into the current token.
    ///
    /// Returns the consumed character.
    pub fn consume(&mut self) -> Option<char> {
        let next = self.iterator.next()?;
        self.length += next.len_utf8();
        Some(next)
    }

    /// If the next character matches some predicate, consume it into the current token.
    ///
    /// Returns the number of characters consumed.
    pub fn consume_while(&mut self, predicate: impl Fn(char) -> bool) -> usize {
        // TODO: Evaluate whether to return usize or ()
        let mut consumed = 0;
        while self.peek().is_some_and(|next| predicate(next)) {
            self.consume();
            consumed += 1;
        }
        consumed
    }

    /// Peek the next character without consuming it.
    pub fn peek(&self) -> Option<char> {
        self.peek_at_offset(0)
    }

    /// Peek the character at the given offset without consuming it.
    pub fn peek_at_offset(&self, offset: usize) -> Option<char> {
        self.iterator.clone().nth(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_lexeme_without_consuming() {
        let mut cursor = Cursor::new("");
        assert_eq!(cursor.close(), Lexeme { start_offset: 0, text: "" });
    }

    #[test]
    fn test_close_lexeme_after_consuming_empty_text() {
        let mut cursor = Cursor::new("");
        assert_eq!(cursor.consume(), None);
        assert_eq!(cursor.close(), Lexeme { start_offset: 0, text: "" });
    }

    #[test]
    fn test_close_lexeme() {
        let mut cursor = Cursor::new("abc");
        assert_eq!(cursor.consume(), Some('a'));
        assert_eq!(cursor.close(), Lexeme { start_offset: 0, text: "a" });
    }

    #[test]
    fn test_consume_while() {
        let mut cursor = Cursor::new("aaabc");
        assert_eq!(cursor.consume_while(|next| next == 'a'), 3);
        assert_eq!(cursor.close(), Lexeme { start_offset: 0, text: "aaa" });
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

    #[test]
    fn test_emoji() {
        let mut cursor = Cursor::new("👨‍👩‍👧‍👦");
        cursor.consume_while(|_| true);
        // We currently don't handle multiple characters joined together.
        // This might change in the future.
        assert_eq!(cursor.close(), Lexeme { start_offset: 0, text: "👨‍👩‍👧‍👦" });
    }
}