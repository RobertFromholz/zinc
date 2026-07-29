//! The concrete syntax tree (CST) is a one-to-one representation of the source code.
//!
//! The parser registers symbols in the source code to a symbol table.

pub mod lexer;
pub mod parser;

pub mod token;
pub mod tree;

use std::fmt;
use std::ops::RangeBounds;
use token::{Token, TokenKind};

/// A substring in the source code.
///
/// Used by nodes in the CST to reference what text in the source code they represent.
#[derive(Clone, PartialEq, Eq)]
pub struct Span<'text> {
    text: &'text str,
    start_offset: usize,
    length: usize
}

impl<'text> Span<'text> {
    pub fn new(text: &'text str, range: impl RangeBounds<usize>) -> Self {
        let start_offset = match range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end_offset = match range.end_bound() {
            std::ops::Bound::Included(&end) => end + 1,
            std::ops::Bound::Excluded(&end) => end,
            std::ops::Bound::Unbounded => text.len(),
        };
        let length = end_offset - start_offset;
        assert!(start_offset <= end_offset);
        assert!(length <= text.len());
        Self {
            text,
            start_offset,
            length
        }
    }

    pub fn text(&self) -> &str {
        &self.text[self.start_offset()..self.end_offset()]
    }

    pub fn start_offset(&self) -> usize {
        self.start_offset
    }

    pub fn end_offset(&self) -> usize {
        self.start_offset() + self.length()
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

impl<'text> fmt::Debug for Span<'text> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Span")
            .field("text", &self.text())
            .field("start_offset", &self.start_offset())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_span() {
        let text = "Hello, World!";
        let span = Span::new(text, 0..5);
        assert_eq!(
            Span {
                text,
                start_offset: 0,
                length: 5
            },
            span
        );
        assert_eq!(span.text(), "Hello");
    }

    #[test]
    fn test_create_span_full_bound() {
        let text = "Hello, World!";
        let span = Span::new(text, ..);
        assert_eq!(
            Span {
                text,
                start_offset: 0,
                length: text.len()
            },
            span
        );
        assert_eq!(span.text(), "Hello, World!");
    }

    #[test]
    fn test_create_span_upper_bound() {
        let text = "Hello, World!";
        let span = Span::new(text, 7..);
        assert_eq!(
            Span {
                text,
                start_offset: 7,
                length: text.len() - 7
            },
            span
        );
        assert_eq!(span.text(), "World!");
    }

    #[test]
    fn test_create_span_lower_bound() {
        let text = "Hello, World!";
        let span = Span::new(text, ..5);
        assert_eq!(
            Span {
                text,
                start_offset: 0,
                length: 5
            },
            span
        );
        assert_eq!(span.text(), "Hello");
    }

    #[test]
    fn test_create_span_inclusive_bound() {
        let text = "Hello, World!";
        let span = Span::new(text, 2..=4);
        assert_eq!(
            Span {
                text,
                start_offset: 2,
                length: 3
            },
            span
        );
        assert_eq!(span.text(), "llo");
    }
}
