use crate::lexer::Token;

pub type Result<'s, T> = std::result::Result<T, ParseError>;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    ExpectedToken { expected: Token, found: Token },
    UnsupportedToken(Token),
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::ExpectedToken { expected, found } => {
                write!(f, "expected {:?}, but found {:?}", expected, found)
            }
            Self::UnsupportedToken(t) => write!(f, "unsupported token: {:?}", t),
        }
    }
}
