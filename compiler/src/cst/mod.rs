//! The concrete syntax tree (CST) is a one-to-one representation of the source code.
//!
//! The parser registers symbols in the source code to a symbol table.

mod lexer;
mod parser;

mod token;
mod tree;

use std::borrow::Borrow;
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
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn start_offset(&self) -> usize {
        self.start_offset
    }

    pub fn end_offset(&self) -> usize {
        self.start_offset() + self.length()
    }

    pub fn length(&self) -> usize {
        self.text().len()
    }

    /// Combine a list of consecutive spans into a new span.
    ///
    /// Returns `None` if the iterator is empty or if the iterator is non-consecutive.
    pub fn combine(spans: impl IntoIterator<Item=impl Borrow<Span>>) -> Option<Span> {
        let mut iter = spans.into_iter();
        // We need to clone the first span.
        // This becomes our return value, all other spans are 'moved' into this span.
        let first = iter.next()?.borrow().clone();
        iter.try_fold(first, |mut span, next| {
            let next = next.borrow();
            if span.end_offset() == next.start_offset() {
                span.text += &next.text;
                Some(span)
            } else {
                None
            }
        })
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
        assert_eq!(None, Span::combine(Vec::<Span>::new()));
    }
}
