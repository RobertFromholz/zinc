mod event;

use crate::cst::lexer::Lexer;
use crate::cst::parser::event::{CloseMarker, EventStream, OpenMarker};
use crate::cst::token::TokenKind;
use crate::cst::tree::{Tree, TreeKind};

/// A parser to convert a stream of tokens into a concrete syntax tree.
pub struct Parser<'text> {
    lexer: Lexer<'text>,
    events: EventStream<'text>,
}

impl<'text> Parser<'text> {
    pub fn new(text: &'text str) -> Self {
        Self {
            lexer: Lexer::new(text),
            events: EventStream::new(),
        }
    }

    pub fn finish(self) -> Tree<'text> {
        self.events.build()
    }

    /// Open a new node.
    fn open(&mut self) -> OpenMarker {
        // We don't want to start a token with whitespace.
        // However, we can't do this if we haven't opened a node yet.
        if !self.events.is_empty() {
            if let Some(token) = self.lexer.next_kind(TokenKind::Whitespace) {
                self.events.consume(token);
            }
        }
        self.events.open()
    }

    /// Open a new node surrounding the specified node.
    fn open_before(&mut self, marker: CloseMarker) -> OpenMarker {
        self.events.open_before(marker)
    }

    /// Close the current node.
    fn close(&mut self, marker: OpenMarker, kind: TreeKind) {
        self.events.close(marker, kind);
    }

    /// Consumes the next node if it is of the provided type.
    ///
    /// Automatically skips any whitespace tokens.
    fn consume(&mut self, kind: TokenKind) {
        self.consume_whitespace();
        match self.lexer.next_kind(kind) {
            Some(token) => self.events.consume(token),
            None => {
                let actual = self.lexer.peek();
                panic!("expected {}, got {:?}", kind, actual);
            }
        }
    }

    /// Consume all whitespace tokens.
    fn consume_whitespace(&mut self) {
        while let Some(token) = self.lexer.peek() {
            if token.kind != TokenKind::Whitespace {
                return;
            }
            let next = self.lexer.next().unwrap();
            self.events.consume(next);
        }
    }

    /// Returns the next node if it is of a specific type.
    fn peek(&mut self, kind: TokenKind) -> Option<TokenKind> {
        self.lexer.peek_kind(kind)
            .map(|token| token.kind)
    }

    /// Returns the next node if it is of a specific type.
    fn peek_at_offset(&mut self, kind: TokenKind, offset: usize) -> Option<TokenKind> {
        self.lexer.peek_kind_at_offset(kind, offset)
            .map(|token| token.kind)
    }

    /// Returns whether the next token is of a specific type.
    fn at(&mut self, kind: TokenKind) -> bool {
        self.lexer.peek_kind(kind).is_some()
    }
}