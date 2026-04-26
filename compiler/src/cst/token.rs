use std::fmt;
use std::ops::RangeBounds;
use crate::cst::Span;

/// A token is a character or sequence in the source code of some associated type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'text> {
    pub(super) kind: TokenKind,
    pub(super) span: Span<'text>,
}

impl<'text> Token<'text> {
    pub fn new(text: &'text str, range: impl RangeBounds<usize>, kind: TokenKind) -> Self {
        Self {
            kind,
            span: Span::new(text, range),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Any sequence of whitespace.
    Whitespace,

    /// An identifier.
    Identifier,

    /// An integer literal.
    Integer,

    /// `class`
    Class,

    /// `let`
    Field,

    /// `function`
    Function,

    /// `constant`
    Constant,

    /// `mutable`
    Mutable,

    /// `,`
    Comma,
    /// `;`
    Semicolon,
    /// `:`
    Colon,
    /// `=`
    Equals,

    // We technically don't use '-' token yet.
    // However, they are used to construct '->'.
    // The lexer shouldn't combine tokens since it doesn't know whether the syntax expects the
    // tokens individually or combined. As a result, it's easier to return a '-' token which can
    // be combined with the '>' token during parsing.

    /// `-`
    Minus,

    /// `<`
    LessThan,

    /// `>`
    GreaterThan,

    /// `->`
    RightArrow,

    /// `::`
    PathSeparator,

    /// `{`
    LeftBrace,

    /// `}`
    RightBrace,

    /// `(`
    LeftParentheses,

    /// `)`
    RightParentheses,

    /// Any unknown character.
    Unknown,
}

impl TokenKind {
    /// Try to combine a list of consecutive tokens into a new token of this type.
    pub fn combine<'text>(self, parts: &[Token<'text>]) -> Option<Token<'text>> {
        let expected = self.decompose().into_iter();
        let actual = parts.iter()
            .map(|token| token.kind);
        if expected.eq(actual) {
            let spans = parts.iter()
                .map(|token| token.span);
            let span = Span::combine(spans)?;
            Some(Token {
                kind: self,
                span,
            })
        } else {
            None
        }
    }

    /// Returns all parts that make up this token.
    pub fn decompose(self) -> Vec<TokenKind> {
        match self {
            TokenKind::RightArrow => vec![TokenKind::Minus, TokenKind::GreaterThan],
            TokenKind::PathSeparator => vec![TokenKind::Colon, TokenKind::Colon],
            _ => vec![self]
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            TokenKind::Whitespace => "whitespace",
            TokenKind::Identifier => "identifier",
            TokenKind::Integer => "integer",
            TokenKind::Class => "class",
            TokenKind::Field => "let",
            TokenKind::Function => "function",
            TokenKind::Constant => "constant",
            TokenKind::Mutable => "mutable",
            TokenKind::Comma => ",",
            TokenKind::Semicolon => ";",
            TokenKind::Colon => ":",
            TokenKind::Equals => "=",
            TokenKind::Minus => "-",
            TokenKind::LessThan => "<",
            TokenKind::GreaterThan => ">",
            TokenKind::RightArrow => "->",
            TokenKind::PathSeparator => "::",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::LeftParentheses => "(",
            TokenKind::RightParentheses => ")",
            TokenKind::Unknown => "unknown"
        })
    }
}

impl TryFrom<&str> for TokenKind {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "class" => Ok(TokenKind::Class),
            "let" => Ok(TokenKind::Field),
            "function" => Ok(TokenKind::Function),
            "constant" => Ok(TokenKind::Constant),
            "mutable" => Ok(TokenKind::Mutable),
            _ => Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_right_arrow() {
        let text = "->";
        let parts = vec![
            Token::new(text, 0..1, TokenKind::Minus),
            Token::new(text, 1..2, TokenKind::GreaterThan),
        ];
        let result = TokenKind::RightArrow.combine(&parts);
        assert_eq!(result, Some(Token::new(text, 0..2, TokenKind::RightArrow)));
    }

    #[test]
    fn test_combine_path_separator() {
        let text = "::";
        let parts = vec![
            Token::new(text, 0..1, TokenKind::Colon),
            Token::new(text, 1..2, TokenKind::Colon),
        ];
        let result = TokenKind::PathSeparator.combine(&parts);
        assert_eq!(result, Some(Token::new(text, 0..2, TokenKind::PathSeparator)));
    }

    #[test]
    fn test_combine_wrong_parts() {
        let text = "-;";
        let parts = vec![
            Token::new(text, 0..1, TokenKind::Minus),
            Token::new(text, 1..2, TokenKind::Semicolon),
        ];
        let result = TokenKind::RightArrow.combine(&parts);
        assert_eq!(result, None);
    }

    #[test]
    fn test_combine_non_consecutive() {
        let text = "- >";
        let parts = vec![
            Token::new(text, 0..1, TokenKind::Minus),
            Token::new(text, 2..3, TokenKind::GreaterThan),
        ];
        let result = TokenKind::RightArrow.combine(&parts);
        assert_eq!(result, None);
    }

    #[test]
    fn test_combine_single_token() {
        let text = ",";
        let parts = vec![
            Token::new(text, 0..1, TokenKind::Comma),
        ];
        let result = TokenKind::Comma.combine(&parts);
        assert_eq!(result, Some(Token::new(text, 0..1, TokenKind::Comma)));
    }

    #[test]
    fn test_combine_insufficient_parts() {
        let text = "-";
        let parts = vec![
            Token::new(text, 0..1, TokenKind::Minus),
        ];
        let result = TokenKind::RightArrow.combine(&parts);
        assert_eq!(result, None);
    }
}
