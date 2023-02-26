use std::fmt::Display;

use colored::Colorize;

use crate::lexer::{Token, TokenKind};

pub type Result<'s, T> = std::result::Result<T, ParseError<'s>>;

#[derive(Debug, PartialEq)]
pub enum ParseErrorKind<'t> {
    ExpectedToken {
        expected: &'static [TokenKind],
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
    OutOfBoundsCaptureIndex {
        index: &'t str,
        number_of_ordinal_captures: usize,
        position: usize,
    },
    UndeclaredIdentifier {
        ident: &'t str,
        declared: Vec<&'t str>,
        position: usize,
    },
}

impl<'t> ParseErrorKind<'t> {
    fn error_location(&self) -> &usize {
        match &self {
            ParseErrorKind::UnsupportedToken(t) => &t.start,
            ParseErrorKind::ExpectedToken { position, .. } => &position,
            ParseErrorKind::UnexpectedToken { position, .. } => &position,
            ParseErrorKind::UndeclaredIdentifier { position, .. } => &position,
            ParseErrorKind::OutOfBoundsCaptureIndex { position, .. } => &position,
        }
    }
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
            Colon => "colon",
            Rparen => "closing parenthesis",
            Lparen => "opening parenthesis",
            CaptureIndex => "a capture index",
        };
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseError<'t> {
    input: &'t str,
    kind: ParseErrorKind<'t>,
    more: Vec<ParseErrorKind<'t>>,
}

impl<'t> ParseError<'t> {
    pub fn new(input: &'t str, kind: ParseErrorKind<'t>) -> Self {
        Self {
            input,
            kind,
            more: vec![],
        }
    }

    pub fn kind(&self) -> &'t ParseErrorKind {
        &self.kind
    }

    pub fn and(mut self, more: ParseErrorKind<'t>) -> Self {
        self.more.push(more);
        self
    }
}

impl<'t> std::error::Error for ParseError<'t> {}

impl<'t> std::fmt::Display for ParseError<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.input.yellow())?;

        write!(f, "{}", self.kind)?;

        for kind in &self.more {
            write!(f, "\n{kind}")?;
        }

        Ok(())
    }
}

impl<'t> Display for ParseErrorKind<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseErrorKind::*;

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

        match &self {
            ExpectedToken {
                expected,
                found,
                text,
                ..
            } => {
                write!(
                    f,
                    "expected: {}; found a {}, {}",
                    expected
                        .iter()
                        .map(|e| e.description().blue().to_string())
                        .collect::<Vec<String>>()
                        .join(", or "),
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
            OutOfBoundsCaptureIndex {
                index,
                number_of_ordinal_captures,
                ..
            } => write!(
                f,
                "out of bounds capture index {}; only {} unamed capture group(s) declared",
                index.to_string().red(),
                number_of_ordinal_captures.to_string().blue(),
            ),
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
       ($input:literal, $error_kind:expr $(,$more:expr)*) => {
            let input = $input;
            let err = Parser::from(input).parse().unwrap_err();

            assert_eq!(
                err,
                ParseError {
                    input: $input,
                    kind: $error_kind,
                    more: vec![$($more),*]
                }
            );
        };
    }

    #[test]
    fn expecting_identifier() {
        assert_error!(
            "a(:int)",
            ParseErrorKind::ExpectedToken {
                expected: &[TokenKind::Ident, TokenKind::Type],
                found: TokenKind::Colon,
                text: ":",
                position: 2
            }
        );

        assert_error!(
            "a(n:int)->(",
            ParseErrorKind::ExpectedToken {
                expected: &[TokenKind::Ident, TokenKind::CaptureIndex],
                found: TokenKind::End,
                text: "",
                position: 11
            }
        );

        assert_error!(
            "a(n:int)->()",
            ParseErrorKind::ExpectedToken {
                expected: &[Ident, CaptureIndex],
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
                expected: &[Rparen],
                found: End,
                text: "",
                position: 6
            }
        );

        assert_error!(
            "(n:int ",
            ExpectedToken {
                expected: &[Rparen],
                found: Literal,
                text: " ",
                position: 6
            }
        );

        assert_error!(
            "(n:int->(n)",
            ExpectedToken {
                expected: &[Rparen],
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
    fn rejecting_out_of_bounds_capture_index() {
        assert_error!(
            "a->(1)",
            OutOfBoundsCaptureIndex {
                index: "1",
                number_of_ordinal_captures: 0,
                position: 4
            }
        );

        assert_error!(
            "a(int)(dig)->(3)",
            OutOfBoundsCaptureIndex {
                index: "3",
                number_of_ordinal_captures: 2,
                position: 14
            }
        );
    }

    #[test]
    fn expecting_types() {
        assert_error!(
            "t(n:)8",
            super::ParseErrorKind::ExpectedToken {
                expected: &[TokenKind::Type],
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

        assert_error!(
            "(in)",
            ParseErrorKind::ExpectedToken {
                expected: &[Colon],
                text: ")",
                found: Rparen,
                position: 3
            },
            ParseErrorKind::UnsupportedToken(Token {
                kind: TokenKind::Type,
                text: crate::lexer::TokenText::Slice("in"),
                start: 1
            })
        );
    }
}
