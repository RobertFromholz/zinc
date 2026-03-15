//! Internally, the parser first converts from a stream of tokens into a stream of events.
//! Events are then converted into a tree.
//!
//! Each event determines whether to create a new node, close the current node, or append a token
//! into the current node.
//!
//! Using events, we can choose to surround the current node in another node without having to
//! change the order of events.

use crate::cst::token::Token;
use crate::cst::tree::TreeKind;

/// A stream of events.
///
/// Whilst constructing the tree, events are taken from the event stream.
/// Events can be processed in a non-linear order, since an event can reference another event
/// that should occur before it. As a result, we don't know if we have already processed an event.
/// We therefore take the event from the event stream when we process it, so a `None` event
/// has already been processed.
#[derive(Debug)]
pub struct EventStream<'text> {
    events: Vec<Option<Event<'text>>>,
}

#[derive(Debug, PartialEq)]
pub enum Event<'text> {
    /// Start a new node.
    ///
    /// Previous can reference the index of an event that should occur before this event.
    Start {
        kind: TreeKind,
        previous: Option<usize>,
    },

    /// Close the current node.
    Finish,

    /// Append a token to the current node.
    Token { token: Token<'text> },

    /// Register the current node as a symbol in the symbol table.
    ///
    /// This event must occur immediately after a `Token` event, which marks the symbol's
    /// identifier.
    Symbol,

    /// Register an error at this point in the stream.
    ///
    /// This event must occur immediately after a `Token` event, to which this error belongs.
    Error { message: String },
}

impl<'text> EventStream<'text> {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Open a new node. The node's type is determined when it is closed.
    ///
    /// Returns an OpenMarker, used to close the node.
    pub fn open(&mut self) -> OpenMarker {
        self.events.push(Some(Event::Start {
            kind: TreeKind::Unknown,
            previous: None,
        }));
        OpenMarker { index: self.events.len() - 1 }
    }

    /// Open a new node surrounding a node. The node's type is determined when it is closed.
    ///
    /// The created node becomes the parent node's parent.
    pub fn open_before(&mut self, marker: CloseMarker) -> OpenMarker {
        let open = self.open();
        match self.events.get_mut(marker.index) {
            Some(Some(Event::Start { previous, .. })) => *previous = Some(open.index),
            _ => panic!("cannot open node; marker is invalid")
        }
        open
    }

    /// Close the node and determine its type.
    pub fn close(&mut self, marker: OpenMarker, kind: TreeKind) -> CloseMarker {
        // TODO: Verify that 'marker' is the last last opened node.
        //  Otherwise, we will actually be closing some other node.
        match self.events.get_mut(marker.index) {
            Some(Some(Event::Start { kind: current, .. })) => *current = kind.clone(),
            _ => panic!("cannot close node; marker is invalid")
        }
        self.events.push(Some(Event::Finish));
        CloseMarker { index: marker.index }
    }

    /// Consume a token.
    ///
    /// The token is consumed by the previously opened node.
    pub fn consume(&mut self, token: Token<'text>) {
        // TODO: Verify that we have an opened node.
        self.events.push(Some(Event::Token { token }));
    }

    /// Register the current node as a symbol.
    ///
    /// Must be called after consuming the token representing the identifier for this symbol.
    pub fn symbol(&mut self) {
        // TODO: Verify that we have an opened node, and that the last node is an identifier.
        self.events.push(Some(Event::Symbol));
    }

    /// Register an error at the previously consumed token.
    pub fn error(&mut self, message: String) {
        self.events.push(Some(Event::Error { message }));
    }
}

/// A reference to an unclosed node in the tree.
#[must_use]
pub struct OpenMarker {
    index: usize,
}

/// A reference to a closed node in the tree.
pub struct CloseMarker {
    index: usize,
}

impl<'text> IntoIterator for EventStream<'text> {
    type Item = Event<'text>;
    type IntoIter = IntoIter<'text>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            events: self.events,
            stack: vec![0],
        }
    }
}

/// An iterator over all events.
///
/// We need to keep track of multiple events. We need to return the event occuring before
/// the current event, and potientially the event occuring before that event, and so on. Afterward,
/// we need to know which event to continue iterating from.
pub struct IntoIter<'text> {
    events: Vec<Option<Event<'text>>>,
    stack: Vec<usize>,
}

/// Iterate over all events in this stream.
impl<'text> Iterator for IntoIter<'text> {
    type Item = Event<'text>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.stack.pop()?;
        let event = self.events[next]
            .take()
            .unwrap();
        // Calculate the next event.
        // We must do this now, because we don't store the index of the current event.
        if self.stack.is_empty() {
            // Find the next unprocessed event.
            let mut next = next + 1;
            let next = loop {
                match self.events.get(next) {
                    // This event has not yet been processed.
                    Some(Some(_)) => break Some(next),
                    // This event has already been processed.
                    // Try to process the next event.
                    Some(_) => next += 1,
                    // We've reached the end of the stream.
                    None => break None,
                }
            };
            if let Some(mut next) = next {
                self.stack.push(next);
                // Add all events that should occur before this event to the stack.
                while let Some(Some(Event::Start { previous: Some(previous), .. })) = self.events.get(next) {
                    next = *previous;
                    self.stack.push(next);
                }
            }
        }
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::Span;
    use crate::cst::token::TokenKind;

    #[test]
    fn test_iterate_events() {
        let mut stream = EventStream::new();
        let marker = stream.open();
        stream.consume(Token { kind: TokenKind::Identifier, span: Span { text: "abc", start_offset: 0, length: 0 } });
        stream.close(marker, TreeKind::File);
        assert_eq!(
            vec![
                Event::Start { kind: TreeKind::File, previous: None },
                Event::Token { token: Token { kind: TokenKind::Identifier, span: Span { text: "abc", start_offset: 0, length: 0 } } },
                Event::Finish
            ],
            stream.into_iter().collect::<Vec<Event>>()
        )
    }
}
