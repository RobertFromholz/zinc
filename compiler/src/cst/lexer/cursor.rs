//! A `Cursor` reads source code and converts it into a stream of lexeme.
//! 
//! A `Lexeme` represents the position of a token. Unlike a token, a lexeme does not have a type. 
//! The type is determined by the lexer, which converts the lexeme into a token.
//! 
//! The lexer instructs the cursor when to consume characters and when to start a new lexeme.

use std::str::Chars;
use crate::cst::Span;

/// A `Cursor` reads source code and converts it into a stream of lexeme.
/// 
/// The cursor knows where it is in the source code, as-well as where the current lexeme started 
/// and how long it is. It does not, however, know anything about how to tokenize the source code.
/// The lexer is responsible for instructing the cursor when to consume characters and when to
/// start a new lexeme. 
pub struct Cursor<'text> {
    text: &'text str,
    iterator: Chars<'text>,
    start_offset: usize,
    length: usize,
}

impl<'text> Cursor<'text> {
    /// Create a new `Cursor` to read the given text.
    pub fn new(text: &'text str) -> Self {
        Self {
            text,
            iterator: text.chars(),
            start_offset: 0,
            length: 0,
        }
    }

    /// Returns the current lexeme.
    pub fn current(&self) -> Span {
        Span::new(self.text, self.start_offset..self.start_offset + self.length)
    }

    /// Close the current lexeme. The current lexeme is returned.
    pub fn close(&mut self) -> Span {
        let current = self.current();
        self.start_offset += self.length;
        self.length = 0;
        current
    }

    /// Consume the next character into the current lexeme.
    ///
    /// Returns the consumed character.
    pub fn consume(&mut self) -> Option<char> {
        let next = self.iterator.next()?;
        self.length += next.len_utf8();
        Some(next)
    }

    /// If the next character matches the given predicate, consume it into the current lexeme.
    ///
    /// Returns the number of characters consumed by the predicate.
    pub fn consume_while(&mut self, predicate: impl Fn(char) -> bool) -> usize {
        // TODO: Evaluate whether to return usize or ()
        let mut consumed = 0;
        while self.peek().is_some_and(|next| predicate(next)) {
            self.consume();
            consumed += 1;
        }
        consumed
    }

    /// Return the next character without consuming it.
    pub fn peek(&self) -> Option<char> {
        self.peek_at_offset(0)
    }

    /// Return the character at the given offset without consuming it.
    /// 
    /// An offset of `0` indicates the next character to be consumed.
    pub fn peek_at_offset(&self, offset: usize) -> Option<char> {
        self.iterator.clone().nth(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_lexeme_without_consuming() {
        let text = "";
        let mut cursor = Cursor::new(text);
        assert_eq!(cursor.close(), Span::new(text, 0..0));
    }

    #[test]
    fn test_close_lexeme_after_consuming_empty_text() {
        let text = "";
        let mut cursor = Cursor::new(text);
        assert_eq!(cursor.consume(), None);
        assert_eq!(cursor.close(), Span::new(text, 0..0));
    }

    #[test]
    fn test_close_lexeme() {
        let text = "abc";
        let mut cursor = Cursor::new(text);
        assert_eq!(cursor.consume(), Some('a'));
        assert_eq!(cursor.close(), Span::new(text, 0..1));
    }

    #[test]
    fn test_consume_while() {
        let text = "aaabc";
        let mut cursor = Cursor::new(text);
        assert_eq!(cursor.consume_while(|next| next == 'a'), 3);
        assert_eq!(cursor.close(), Span::new(text, 0..3));
    }

    #[test]
    fn test_peek() {
        let text = "abc";
        let mut cursor = Cursor::new(text);
        assert_eq!(cursor.peek(), Some('a'));
        assert_eq!(cursor.peek(), Some('a'));
        assert_eq!(cursor.consume(), Some('a'));
        assert_eq!(cursor.peek(), Some('b'));
        assert_eq!(cursor.close(), Span::new( text, 0..1))
    }

    #[test]
    fn test_peek_at_offset() {
        let text = "abc";
        let mut cursor = Cursor::new(text);
        assert_eq!(cursor.peek_at_offset(0), Some('a'));
        assert_eq!(cursor.peek_at_offset(1), Some('b'));
        assert_eq!(cursor.consume(), Some('a'));
        assert_eq!(cursor.peek_at_offset(0), Some('b'));
        assert_eq!(cursor.peek_at_offset(1), Some('c'));
        assert_eq!(cursor.close(), Span::new(text, 0..1))
    }

    #[test]
    fn test_emoji() {
        let text = "👨‍👩‍👧‍👦";
        let mut cursor = Cursor::new(text);
        cursor.consume_while(|_| true);
        // We currently don't handle multiple characters joined together.
        // This might change in the future.
        assert_eq!(cursor.close(), Span::new(text, 0..));
    }
}
