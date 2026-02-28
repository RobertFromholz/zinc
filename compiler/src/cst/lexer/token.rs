/// A token is a specific sequence in the source code with an associated type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'text> {
    pub(super) kind: TokenKind,
    /// The starting offset (in Unicode characters) of the token.
    pub(super) start_offset: usize,
    /// The length (in Unicode characters) of the token.
    pub(super) length: usize,
    pub(super) text: &'text str,
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
    Unknown
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

impl From<KeywordKind> for &str {
    fn from(value: KeywordKind) -> Self {
        match value {
            KeywordKind::Module => "module",
            KeywordKind::Class => "class",
            KeywordKind::Field => "let",
            KeywordKind::Function => "function",
            KeywordKind::Constant => "constant",
            KeywordKind::Mutable => "mutable"
        }
    }
}