use std::fmt;
use std::fmt::Formatter;

/// A token is a character or sequence in the source code of some associated type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Token {
    pub(super) kind: TokenKind,
    pub(super) length: usize,
}

impl Token {
    pub fn new(kind: TokenKind, length: usize) -> Self {
        Self {
            kind,
            length
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{} [{}]", self.kind, self.length)
        } else {
            write!(f, "{}", self.kind)
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

    /// `struct`
    Struct,

    /// `let`
    Let,

    /// `fn`
    Fn,

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
    pub fn combine(self, parts: &[Token]) -> Option<Token> {
        let expected = self.decompose().into_iter();
        let actual = parts.iter()
            .map(|token| token.kind);
        if expected.eq(actual) {
            let length = parts.iter()
                .map(|token| token.length)
                .sum();
            Some(Token {
                kind: self,
                length,
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
            TokenKind::Struct => "struct",
            TokenKind::Let => "let",
            TokenKind::Fn => "fn",
            TokenKind::Comma => ",",
            TokenKind::Semicolon => ";",
            TokenKind::Colon => ":",
            TokenKind::Equals => "=",
            TokenKind::Minus => "-",
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
            "struct" => Ok(TokenKind::Struct),
            "let" => Ok(TokenKind::Let),
            "fn" => Ok(TokenKind::Fn),
            _ => Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_right_arrow() {
        let parts = vec![
            Token::new(TokenKind::Minus, 1),
            Token::new(TokenKind::GreaterThan, 1),
        ];
        let result = TokenKind::RightArrow.combine(&parts);
        assert_eq!(result, Some(Token::new(TokenKind::RightArrow, 2)));
    }

    #[test]
    fn test_combine_path_separator() {
        let parts = vec![
            Token::new(TokenKind::Colon, 1),
            Token::new(TokenKind::Colon, 1),
        ];
        let result = TokenKind::PathSeparator.combine(&parts);
        assert_eq!(result, Some(Token::new(TokenKind::PathSeparator, 2)));
    }

    #[test]
    fn test_combine_wrong_parts() {
        let parts = vec![
            Token::new(TokenKind::Minus, 1),
            Token::new(TokenKind::Semicolon, 1),
        ];
        let result = TokenKind::RightArrow.combine(&parts);
        assert_eq!(result, None);
    }

    #[test]
    fn test_combine_non_consecutive() {
        let parts = vec![
            Token::new(TokenKind::Minus, 1),
            Token::new(TokenKind::Whitespace, 1),
            Token::new(TokenKind::GreaterThan, 1),
        ];
        let result = TokenKind::RightArrow.combine(&parts);
        assert_eq!(result, None);
    }

    #[test]
    fn test_combine_single_token() {
        let parts = vec![
            Token::new(TokenKind::Comma, 1),
        ];
        let result = TokenKind::Comma.combine(&parts);
        assert_eq!(result, Some(Token::new(TokenKind::Comma, 1)));
    }

    #[test]
    fn test_combine_insufficient_parts() {
        let parts = vec![
            Token::new(TokenKind::Minus, 1),
        ];
        let result = TokenKind::RightArrow.combine(&parts);
        assert_eq!(result, None);
    }
}
