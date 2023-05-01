use colored::Colorize;

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
    UnexpectedToken {
        unexpected: TokenKind,
        previous: TokenKind,
        position: usize,
    },
    UndeclaredIdentifier {
        ident: &'t str,
        declared: Vec<&'t str>,
        position: usize,
    },
}

impl TokenKind {
    fn description(&self) -> &str {
        use TokenKind::*;

        return match self {
            Literal => "literal",
            Type => "type keyword",
            Ident => "identifier",
            Arrow => "pattern seperator",
            End => "end of expression",
            _ => "special character",
        };
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseError<'t> {
    pub(crate) input: &'t str,
    pub(crate) kind: ParseErrorKind<'t>,
}

impl<'t> ParseError<'t> {
    fn error_location(&self) -> &usize {
        match &self.kind {
            ParseErrorKind::UnsupportedToken(t) => &t.start,
            ParseErrorKind::ExpectedToken { position, .. } => &position,
            ParseErrorKind::UnexpectedToken { position, .. } => &position,
            ParseErrorKind::UndeclaredIdentifier { position, .. } => &position,
        }
    }
}

impl<'t> std::error::Error for ParseError<'t> {}

impl<'t> std::fmt::Display for ParseError<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseErrorKind::*;

        writeln!(f, "\n{}", self.input.yellow())?;

        let location = self.error_location();

        for _ in 0..*location {
            write!(f, " ")?;
        }

        write!(
            f,
            "{} {}:{} ",
            "\u{21B3}".red().bold(),
            "@col".red().bold(),
            location.to_string().bold()
        )?;

        match &self.kind {
            ExpectedToken {
                expected,
                found,
                text,
                ..
            } => {
                write!(
                    f,
                    "expected {}, but found a {}, {}",
                    expected.description().blue(),
                    found.description().red(),
                    format!("\"{text}\"").yellow()
                )
            }
            UnsupportedToken(t) => {
                let result = write!(
                    f,
                    "unsupported token: {} {}",
                    t.kind.description().red(),
                    format!("\"{}\"", t.text).yellow()
                );

                if let TokenKind::Type = t.kind {
                    return write!(
                        f,
                        " - supported types are: {}, {}",
                        "int".purple(),
                        "dig".purple()
                    );
                }

                result
            }
            UnexpectedToken {
                unexpected,
                previous,
                ..
            } => {
                write!(
                    f,
                    "unexpected {}, after a {}",
                    unexpected.description().red(),
                    previous.description().blue()
                )
            }
            UndeclaredIdentifier {
                ident, declared, ..
            } => {
                write!(
                    f,
                    "undeclared identifier {}; declared: {}",
                    ident.to_string().red(),
                    declared
                        .iter()
                        .map(|i| i.blue().to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use ParseErrorKind::*;
    use TokenKind::*;

    macro_rules! assert_error {
        ($input:literal, $error_kind:expr) => {
            let input = $input;
            let err = Parser::from(input).parse().unwrap_err();

            assert_eq!(
                err,
                ParseError {
                    input: $input,
                    kind: $error_kind
                }
            );
        };
    }

    #[test]
    fn expecting_identifier() {
        assert_error!(
            "a(:int)",
            ParseErrorKind::ExpectedToken {
                expected: TokenKind::Ident,
                found: TokenKind::Colon,
                text: ":",
                position: 2
            }
        );

        assert_error!(
            "a(n:int)->(",
            ParseErrorKind::ExpectedToken {
                expected: TokenKind::Ident,
                found: TokenKind::End,
                text: "",
                position: 11
            }
        );

        assert_error!(
            "a(n:int)->()",
            ParseErrorKind::ExpectedToken {
                expected: TokenKind::Ident,
                found: TokenKind::Rparen,
                text: ")",
                position: 11
            }
        );
    }

    #[test]
    fn expecting_capture_closing_paren() {
        assert_error!(
            "(n:int",
            ExpectedToken {
                expected: Rparen,
                found: End,
                text: "",
                position: 6
            }
        );

        assert_error!(
            "(n:int ",
            ExpectedToken {
                expected: Rparen,
                found: Literal,
                text: " ",
                position: 6
            }
        );

        assert_error!(
            "(n:int->(n)",
            ExpectedToken {
                expected: Rparen,
                found: Arrow,
                text: "->",
                position: 6
            }
        );
    }

    #[test]
    fn expecting_replacement_exp_after_arrow() {
        assert_error!(
            "wer324->",
            UnexpectedToken {
                unexpected: End,
                previous: Arrow,
                position: 8
            }
        );
    }

    #[test]
    fn rejecting_undeclared_identifers() {
        assert_error!(
            "a->(n)",
            UndeclaredIdentifier {
                ident: "n",
                declared: vec![],
                position: 4
            }
        );

        assert_error!(
            "a(a:int)(ell:dig)->(n)",
            UndeclaredIdentifier {
                ident: "n",
                declared: vec!["a", "ell"],
                position: 20
            }
        );
    }

    #[test]
    fn expecting_types() {
        assert_error!(
            "t(n:)8",
            super::ParseErrorKind::ExpectedToken {
                expected: TokenKind::Type,
                found: TokenKind::Rparen,
                text: ")",
                position: 4
            }
        );

        assert_error!(
            "t(n:di)8",
            ParseErrorKind::UnsupportedToken(Token {
                kind: TokenKind::Type,
                text: crate::lexer::TokenText::Slice("di"),
                start: 4
            })
        );
    }
}
