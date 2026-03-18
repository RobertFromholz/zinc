//! The concrete syntax tree (CST) is a one-to-one representation of the source code.
//!
//! The parser registers symbols in the source code to a symbol table.

mod lexer;
mod parser;

mod token;
mod tree;

use std::ops::RangeBounds;
use token::{Token, TokenKind};
use tree::{Node, Tree, TreeKind};

/// A substring in the source code.
///
/// Used by nodes in the CST to reference what text in the source code they represent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span<'text> {
    text: &'text str,
    start_offset: usize,
    length: usize,
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
        Self {
            text,
            start_offset,
            length: end_offset - start_offset,
        }
    }

    /// Combine a list of consecutive spans into a new span.
    ///
    /// Returns `None` if the iterator is empty or if the iterator is non-consecutive.
    pub fn combine(spans: impl IntoIterator<Item=Span<'text>>) -> Option<Span<'text>> {
        let mut iter = spans.into_iter();
        let first = iter.next()?;
        iter.try_fold(first, |previous, next| {
            if previous.end_offset() == next.start_offset() && previous.text == next.text {
                Some(Span {
                    text: previous.text,
                    start_offset: previous.start_offset,
                    length: previous.length + next.length,
                })
            } else {
                None
            }
        })
    }

    pub fn text(self) -> &'text str {
        &self.text[self.start_offset..self.start_offset + self.length]
    }

    pub fn start_offset(self) -> usize {
        self.start_offset
    }

    pub fn end_offset(self) -> usize {
        self.start_offset + self.length
    }

    pub fn length(self) -> usize {
        self.length
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
                text,
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

    #[test]
    fn test_combine_one_span() {
        let text = "abc";
        assert_eq!(
            Some(Span::new(text, 0..3)),
            Span::combine(vec![
                Span::new(text, 0..3)
            ])
        )
    }

    #[test]
    fn test_combine_consecutive_spans() {
        assert_eq!(
            Some(Span::new("abc123", 0..6)),
            Span::combine(vec![
                Span::new("abc123", 0..3),
                Span::new("abc123", 3..6)
            ])
        )
    }

    #[test]
    fn test_combine_non_consecutive_spans() {
        assert_eq!(
            None,
            Span::combine(vec![
                Span::new("abc 123", 0..3),
                Span::new("abc 123", 4..7)
            ])
        )
    }

    #[test]
    fn test_combine_empty_spans() {
        assert_eq!(None, Span::combine(vec![]));
    }
}
