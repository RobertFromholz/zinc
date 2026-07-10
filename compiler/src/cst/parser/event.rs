//! Internally, the parser first converts from a stream of tokens into a stream of events.
//! Events are then converted into a tree.
//!
//! Each event determines whether to create a new node, close the current node, or append a token
//! into the current node.
//!
//! Using events, we can choose to surround the current node in another node without having to
//! change the order of events.

use crate::cst::token::Token;
use crate::cst::tree::{Node, Tree, TreeKind};

/// A stream of events.
///
/// Whilst constructing the tree, events are taken from the event stream.
/// Events can be processed in a non-linear order, since an event can reference another event
/// that should occur before it. As a result, we don't know if we have already processed an event.
/// We therefore take the event from the event stream when we process it, so a `None` event
/// has already been processed.
///
/// We don't validate events during parsing. Currently, any error is raised when the event stream
/// is converted into a tree.
#[derive(Debug)]
pub struct EventStream {
    events: Vec<Option<Event>>,
}

#[derive(Debug, PartialEq)]
pub enum Event {
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
    Token { token: Token },

    /// Register an error at this point in the stream.
    ///
    /// This event must occur immediately after a `Token` event, to which this error belongs.
    Error { message: String },
}

impl EventStream {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn build(self) -> Tree {
        let mut offset = 0;
        let mut stack = Vec::new();
        let mut iter = self.into_iter();
        
        for event in &mut iter {
            match event {
                Event::Start { kind, .. } => stack.push(Tree {
                    kind,
                    start_offset: offset,
                    children: Vec::new(),
                }),
                Event::Finish => {
                    let node = stack.pop()
                        .expect("unexpected 'Event::Finish' without corresponding 'Event::Start' event");
                    match stack.last_mut() {
                        Some(parent) => {
                            parent.children.push(Node::Tree(node))
                        }
                        None => {
                            if let Some(event) = iter.next() {
                                // We are trying to parse an event, but we have already closed
                                // the top level-node (the file).
                                panic!("unexpected '{:?}' outside top-level node", event);
                            }
                            return node;
                        }
                    }
                }
                Event::Token { token } => {
                    match stack.last_mut() {
                        Some(parent) => {
                            offset = token.span.end_offset();
                            parent.children.push(Node::Token(token));
                        }
                        None => {
                            panic!("unexpected '{:?}' outside top-level node", Event::Token { token });
                        }
                    }
                }
                Event::Error { message } => {
                    let node = stack.last_mut()
                        .expect("unexpected 'Event::Error' without corresponding 'Event::Start' event");
                    node.children.push(Node::Error(message));
                }
            }
        }
        panic!("expected 'Event::Finish'")
    }

    /// Returns whether this event stream is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
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
    pub fn consume(&mut self, token: Token) {
        self.events.push(Some(Event::Token { token }));
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

impl<'text> IntoIterator for EventStream {
    type Item = Event;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            events: self.events,
            stack: Vec::new(),
        }
    }
}

/// An iterator over all events.
///
/// The iterator automatically orders events correctly. As a result, you do not need to worry
/// about the `previous` field for `Event::Start` events.
///
/// We need to keep track of multiple events. We need to return the event occuring before
/// the current event, and potientially the event occuring before that event, and so on. Afterward,
/// we need to know which event to continue iterating from.
pub struct IntoIter {
    events: Vec<Option<Event>>,
    stack: Vec<usize>,
}

/// Iterate over all events in this stream.
impl<'text> Iterator for IntoIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.stack.pop();
        // Calculate the next event.
        // We must do this now, because we don't store the index of the current event.
        if self.stack.is_empty() || index.is_none() {
            let mut next = index.map(|i| i + 1).unwrap_or(0);
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
        let index = match index {
            None => self.stack.pop()?,
            Some(index) => index,
        };
        let event = self.events[index]
            .take()
            .unwrap();
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::token::TokenKind;

    fn compare_event_stream(build: impl FnOnce(&mut EventStream), events: Vec<Event>) {
        let mut stream = EventStream::new();
        build(&mut stream);
        assert_eq!(stream.into_iter().collect::<Vec<_>>(), events);
    }

    #[test]
    fn test_iterate_events() {
        compare_event_stream(|stream| {
            let marker = stream.open();
            stream.consume(Token::new("abc", 0..3, TokenKind::Identifier));
            stream.close(marker, TreeKind::File);
        }, vec![
            Event::Start { kind: TreeKind::File, previous: None },
            Event::Token { token: Token::new("abc", 0..3, TokenKind::Identifier) },
            Event::Finish
        ]);
    }

    #[test]
    fn test_iterate_non_linear_event() {
        compare_event_stream(|stream| {
            let marker = stream.open();
            stream.consume(Token::new("abcdef", 0..3, TokenKind::Identifier));
            let marker = stream.close(marker, TreeKind::Module);
            let marker = stream.open_before(marker);
            stream.consume(Token::new("abcdef", 3..6, TokenKind::Identifier));
            stream.close(marker, TreeKind::File);
        }, vec![
            Event::Start { kind: TreeKind::File, previous: None },
            Event::Start { kind: TreeKind::Module, previous: Some(3) },
            Event::Token { token: Token::new("abcdef", 0..3, TokenKind::Identifier) },
            Event::Finish,
            Event::Token { token: Token::new("abcdef", 3..6, TokenKind::Identifier) },
            Event::Finish,
        ])
    }

    #[test]
    fn test_iterate_multiple_non_linear_events() {
        compare_event_stream(|stream| {
            let marker = stream.open();
            let marker = stream.close(marker, TreeKind::Function);
            let marker = stream.open_before(marker);
            let marker = stream.close(marker, TreeKind::Struct);
            let marker = stream.open_before(marker);
            let marker = stream.close(marker, TreeKind::Module);
            let marker = stream.open_before(marker);
            stream.close(marker, TreeKind::File);
        }, vec![
            Event::Start { kind: TreeKind::File, previous: None },
            Event::Start { kind: TreeKind::Module, previous: Some(6) },
            Event::Start { kind: TreeKind::Struct, previous: Some(4) },
            Event::Start { kind: TreeKind::Function, previous: Some(2) },
            Event::Finish,
            Event::Finish,
            Event::Finish,
            Event::Finish
        ])
    }

    #[test]
    fn build_event_stream() {
        let mut stream = EventStream::new();
        let marker = stream.open();
        stream.consume(Token::new("abc", 0..3, TokenKind::Identifier));
        stream.close(marker, TreeKind::File);
        let tree = stream.build();
        assert_eq!(
            Tree {
                kind: TreeKind::File,
                start_offset: 0,
                children: vec![
                    Node::Token(Token::new("abc", 0..3, TokenKind::Identifier))
                ]
            },
            tree
        );
    }
}
