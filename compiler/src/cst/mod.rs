//! The concrete syntax tree (CST) is a one-to-one representation of the source code.
//!
//! The parser registers symbols in the source code to a symbol table.

mod lexer;
mod parser;

mod token;
mod tree;

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
    fn test_combine_one_span() {
        assert_eq!(
            Some(Span {
                text: "abc",
                start_offset: 0,
                length: 3
            }),
            Span::combine(vec![
                Span {
                    text: "abc",
                    start_offset: 0,
                    length: 3
                }
            ])
        )
    }

    #[test]
    fn test_combine_consecutive_spans() {
        assert_eq!(
            Some(Span {
                text: "abc123",
                start_offset: 0,
                length: 6
            }),
            Span::combine(vec![
                Span {
                    text: "abc123",
                    start_offset: 0,
                    length: 3
                },
                Span {
                    text: "abc123",
                    start_offset: 3,
                    length: 3
                }
            ])
        )
    }

    #[test]
    fn test_combine_non_consecutive_spans() {
        assert_eq!(
            None,
            Span::combine(vec![
                Span {
                    text: "abc 123",
                    start_offset: 0,
                    length: 3
                },
                Span {
                    text: "abc 123",
                    start_offset: 4,
                    length: 3
                }
            ])
        )
    }

    #[test]
    fn test_combine_empty_spans() {
        assert_eq!(None, Span::combine(vec![]));
    }
}
