use crate::lexer::{Token, TokenKind};

pub type Result<'s, T> = std::result::Result<T, ParseError<'s>>;

#[derive(Debug, PartialEq)]
pub enum ParseError<'t> {
    ExpectedToken {
        expected: TokenKind,
        found: Token<'t>,
    },
    UnsupportedToken(Token<'t>),
}

impl<'t> std::error::Error for ParseError<'t> {}

impl<'t> std::fmt::Display for ParseError<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::ExpectedToken { expected, found } => {
                write!(f, "expected {:?}, but found {:?}", expected, found)
            }
            Self::UnsupportedToken(t) => write!(f, "unsupported token: {:?}", t),
        }
    }
}
