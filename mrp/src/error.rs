use std::fmt::Display;

use crate::lexer::{Token, TokenKind};

pub type Result<'s, T> = std::result::Result<T, ParseError<'s>>;

#[derive(Debug, PartialEq)]
pub enum ParseErrorKind<'t> {
    ExpectedToken {
        expected: TokenKind,
        found: TokenKind,
        text: &'t str,
        position: usize,
    },
    UnsupportedToken(Token<'t>),
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenKind::*;

        return match self {
            Literal => write!(f, "literal"),
            DigitType | IntType => write!(f, "type keyword - 'int' or 'dig'"),
            Ident => write!(f, "identifier"),
            Arrow => write!(f, "pattern seperator"),
            End => write!(f, "\0"),
            _ => write!(f, "special character"),
        };
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseError<'t> {
    pub(crate) input: &'t str,
    pub(crate) kind: ParseErrorKind<'t>,
}

impl<'t> std::error::Error for ParseError<'t> {}

impl<'t> std::fmt::Display for ParseError<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseErrorKind::*;

        match &self.kind {
            ExpectedToken {
                expected,
                found,
                text,
                position,
            } => {
                writeln!(f, "{}", self.input)?;

                for _ in 0..*position {
                    write!(f, " ")?;
                }

                writeln!(
                    f,
                    "\u{21B3} @col:{position} expected a {expected}, but found a {found}, \"{text}\""
                )
            }
            UnsupportedToken(t) => write!(f, "unsupported token: {:?}", t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn expecting_types() {
        let input = "t(n:)8";
        let err = Parser::from(input).parse().unwrap_err();

        assert_eq!(
            err,
            ParseError {
                input,
                kind: super::ParseErrorKind::ExpectedToken {
                    expected: TokenKind::IntType,
                    found: TokenKind::Rparen,
                    text: ")",
                    position: 4
                }
            }
        );

        let input = "t(n:di)8";
        let err = Parser::from(input).parse().unwrap_err();

        assert_eq!(
            err,
            ParseError {
                input,
                kind: ParseErrorKind::ExpectedToken {
                    expected: TokenKind::IntType,
                    found: TokenKind::Literal,
                    text: "di",
                    position: 4
                }
            }
        )
    }
}
