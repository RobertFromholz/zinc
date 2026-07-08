//! The concrete syntax tree (CST) is a one-to-one representation of the source code.
//!
//! The parser registers symbols in the source code to a symbol table.

mod lexer;
mod parser;

mod token;
mod tree;

use std::fmt;
use std::ops::RangeBounds;
use token::{Token, TokenKind};

/// A substring in the source code.
///
/// Used by nodes in the CST to reference what text in the source code they represent.
#[derive(Clone, PartialEq, Eq)]
pub struct Span {
    text: String,
    start_offset: usize,
    length: usize,
}

impl Span {
    pub fn new(text: &str, range: impl RangeBounds<usize>) -> Self {
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
        Self {
            text: text[start_offset..end_offset].to_owned(),
            start_offset,
            length: end_offset - start_offset,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn start_offset(&self) -> usize {
        self.start_offset
    }

    pub fn end_offset(&self) -> usize {
        self.start_offset + self.length
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Span")
            .field("text", &self.text())
            .field("start_offset", &self.start_offset())
            .field("length", &self.length())
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
                text: "Hello".to_owned(),
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
                text: "Hello, World!".to_owned(),
                start_offset: 0,
                length: 13
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
                text: "World!".to_owned(),
                start_offset: 7,
                length: 6
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
                text: "Hello".to_owned(),
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
                text: "llo".to_owned(),
                start_offset: 2,
                length: 3
            },
            span
        );
        assert_eq!(span.text(), "llo");
    }
}
