use std::fmt;

/// A token is a specific sequence in the source code with an associated type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'text> {
    pub(super) kind: TokenKind,
    pub(super) start_offset: usize,
    pub(super) text: &'text str,
}

impl<'text> Token<'text> {
    /// This token's type.
    pub fn kind(self) -> TokenKind {
        self.kind
    }

    /// The text represented by this token.
    pub fn text(self) -> &'text str {
        self.text
    }

    /// This token's start offset in bytes.
    pub fn start_offset(self) -> usize {
        self.start_offset
    }

    /// This token's end offset in bytes.
    pub fn end_offset(self) -> usize {
        self.start_offset() + self.length()
    }

    /// This token's length in bytes.
    pub fn length(self) -> usize {
        self.text.len()
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

    /// Any keyword.
    Keyword(KeywordKind),

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
    pub fn combine<'text>(self, text: &'text str, parts: &[Token<'text>]) -> Option<Token<'text>> {
        let is_consecutive = parts.windows(2)
            .all(|window| window[0].end_offset() == window[1].start_offset());
        let expected = self.decompose().into_iter();
        let actual = parts.iter()
            .map(|token| token.kind);
        if is_consecutive && expected.eq(actual) {
            let start_offset = parts.first()?.start_offset();
            let end_offset = parts.last()?.end_offset();
            Some(Token {
                kind: self,
                start_offset,
                text: &text[start_offset..end_offset],
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
            TokenKind::Keyword(keyword) => {
                return write!(f, "{}", keyword)
            },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordKind {
    /// `module`
    Module,
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
}

impl TryFrom<&str> for KeywordKind {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "module" => Ok(KeywordKind::Module),
            "class" => Ok(KeywordKind::Class),
            "let" => Ok(KeywordKind::Field),
            "function" => Ok(KeywordKind::Function),
            "constant" => Ok(KeywordKind::Constant),
            "mutable" => Ok(KeywordKind::Mutable),
            _ => Err(())
        }
    }
}

impl fmt::Display for KeywordKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            KeywordKind::Module => "module",
            KeywordKind::Class => "class",
            KeywordKind::Field => "let",
            KeywordKind::Function => "function",
            KeywordKind::Constant => "constant",
            KeywordKind::Mutable => "mutable"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_right_arrow() {
        let text = "->";
        let parts = vec![
            Token { kind: TokenKind::Minus, start_offset: 0, text: "-" },
            Token { kind: TokenKind::GreaterThan, start_offset: 1, text: ">" },
        ];
        let result = TokenKind::RightArrow.combine(text, &parts);
        assert_eq!(result, Some(Token { kind: TokenKind::RightArrow, start_offset: 0, text: "->" }));
    }

    #[test]
    fn test_combine_path_separator() {
        let text = "::";
        let parts = vec![
            Token { kind: TokenKind::Colon, start_offset: 0, text: ":" },
            Token { kind: TokenKind::Colon, start_offset: 1, text: ":" },
        ];
        let result = TokenKind::PathSeparator.combine(text, &parts);
        assert_eq!(result, Some(Token { kind: TokenKind::PathSeparator, start_offset: 0, text: "::" }));
    }

    #[test]
    fn test_combine_wrong_parts() {
        let text = "-;";
        let parts = vec![
            Token { kind: TokenKind::Minus, start_offset: 0, text: "-" },
            Token { kind: TokenKind::Semicolon, start_offset: 1, text: ";" },
        ];
        let result = TokenKind::RightArrow.combine(text, &parts);
        assert_eq!(result, None);
    }

    #[test]
    fn test_combine_non_consecutive() {
        let text = "- >";
        let parts = vec![
            Token { kind: TokenKind::Minus, start_offset: 0, text: "-" },
            Token { kind: TokenKind::GreaterThan, start_offset: 2, text: ">" },
        ];
        let result = TokenKind::RightArrow.combine(text, &parts);
        assert_eq!(result, None);
    }

    #[test]
    fn test_combine_single_token() {
        let text = ",";
        let parts = vec![
            Token { kind: TokenKind::Comma, start_offset: 0, text: "," },
        ];
        let result = TokenKind::Comma.combine(text, &parts);
        assert_eq!(result, Some(Token { kind: TokenKind::Comma, start_offset: 0, text: "," }));
    }

    #[test]
    fn test_combine_insufficient_parts() {
        let text = "-";
        let parts = vec![
            Token { kind: TokenKind::Minus, start_offset: 0, text: "-" },
        ];
        let result = TokenKind::RightArrow.combine(text, &parts);
        assert_eq!(result, None);
    }
}