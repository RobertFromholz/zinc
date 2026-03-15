mod event;

use crate::cst::lexer::Lexer;

/// A parser to convert a stream of tokens into a concrete syntax tree.
pub struct Parser<'text> {
    lexer: Lexer<'text>
}