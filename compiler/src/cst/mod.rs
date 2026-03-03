//! The concrete syntax tree (CST) is a one-to-one representation of the source code.
//!
//! The parser registers symbols in the source code to a symbol table.

mod lexer;
mod token;
mod tree;

use token::{Token, TokenKind, KeywordKind};
use tree::{Tree, Node, TreeKind};

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
    pub fn combine(mut spans: impl Iterator<Item=Span<'text>>) -> Option<Span<'text>> {
        let first = spans.next()?;
        spans.try_fold(first, |previous, next| {
            if previous.end_offset() == next.start_offset() {
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

