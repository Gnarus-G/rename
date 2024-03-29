use std::str::FromStr;

use crate::{
    error::{ParseError, ParseErrorKind, Result},
    lexer::{Lexer, Token, TokenKind},
    Array,
};

#[derive(Debug, PartialEq, Clone)]
pub enum CaptureType {
    Int,
    Digit,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AbstractMatchingExpression<'source> {
    Literal(&'source str),
    Capture {
        identifier: &'source str,
        identifier_type: CaptureType,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum AbstractReplaceExpression<'source> {
    Literal(&'source str),
    Identifier(&'source str),
}

#[derive(Debug, PartialEq)]
pub struct MatchExpression<'source> {
    pub expressions: Vec<AbstractMatchingExpression<'source>>,
}

impl FromStr for MatchExpression<'static> {
    type Err = ParseError<'static>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let input = Box::leak(s.into());
        Parser::new(Lexer::new(input)).parse_match_exp()
    }
}

impl<'source> MatchExpression<'source> {
    pub fn new(expressions: Vec<AbstractMatchingExpression<'source>>) -> Self {
        Self { expressions }
    }

    pub fn get_expression(&self, idx: usize) -> Option<AbstractMatchingExpression<'source>> {
        self.expressions.get(idx).cloned()
    }
}

#[derive(Debug, PartialEq)]
pub struct ReplaceExpression<'source> {
    pub expressions: Array<AbstractReplaceExpression<'source>>,
}

#[derive(Debug, PartialEq)]
pub struct MatchAndReplaceExpression<'source> {
    pub mex: MatchExpression<'source>,
    pub rex: ReplaceExpression<'source>,
}

impl FromStr for MatchAndReplaceExpression<'static> {
    type Err = ParseError<'static>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let input = Box::leak(s.into());
        Parser::new(Lexer::new(input)).parse()
    }
}

pub struct Parser<'source> {
    lexer: Lexer<'source>,
    peeked: Option<Token<'source>>,
}

impl<'source> Parser<'source> {
    pub fn new(lexer: Lexer<'source>) -> Self {
        Self {
            lexer,
            peeked: None,
        }
    }

    fn token(&mut self) -> Token<'source> {
        match self.peeked.take() {
            Some(t) => t,
            None => self.lexer.next_token(),
        }
    }

    fn peek_token(&mut self) -> &Token<'source> {
        self.peeked.get_or_insert_with(|| self.lexer.next_token())
    }

    fn eat_token(&mut self) {
        self.token();
    }

    pub(crate) fn parse_match_exp(&mut self) -> Result<'source, MatchExpression<'source>> {
        let mut expressions = vec![];

        let mut token = self.token();

        use TokenKind::*;

        while token.kind != End {
            if let Lparen = token.kind {
                self.expect(Ident)?;
            }

            let exp = match token.kind {
                Literal => AbstractMatchingExpression::Literal(&token.text),
                Ident => {
                    let exp = self.parse_capture(&token.text)?;
                    self.expect(Rparen)?;
                    exp
                }
                Arrow => {
                    self.expect_not(End, Arrow)?;
                    break;
                }
                _ => {
                    token = self.token();
                    continue;
                }
            };

            expressions.push(exp);

            token = self.token();
        }

        Ok(MatchExpression::new(expressions))
    }

    fn parse_capture(
        &mut self,
        identifier: &'source str,
    ) -> Result<'source, AbstractMatchingExpression<'source>> {
        self.eat_token();

        self.expect(TokenKind::Type)?;

        Ok(AbstractMatchingExpression::Capture {
            identifier,
            identifier_type: match self.token() {
                t if t.kind == TokenKind::Type => match *t.text {
                    "int" => CaptureType::Int,
                    "dig" => CaptureType::Digit,
                    _ => {
                        return Err(ParseError {
                            source: self.lexer.input(),
                            kind: ParseErrorKind::UnsupportedToken(t),
                        })
                    }
                },
                _ => unreachable!("we expected a type token"),
            },
        })
    }

    fn expect(&mut self, token_kind: TokenKind) -> Result<'source, ()> {
        let error_kind = match self.peek_token() {
            t if t.kind == token_kind => return Ok(()),
            t => ParseErrorKind::ExpectedToken {
                expected: token_kind,
                found: t.kind,
                position: t.start,
                text: &t.text,
            },
        };

        Err(ParseError {
            source: self.lexer.input(),
            kind: error_kind,
        })
    }

    fn expect_not(&mut self, token_kind: TokenKind, current: TokenKind) -> Result<'source, ()> {
        let error_kind = match self.peek_token() {
            t if t.kind == token_kind => ParseErrorKind::UnexpectedToken {
                unexpected: token_kind,
                previous: current,
                position: t.start,
            },
            _ => return Ok(()),
        };

        Err(ParseError {
            source: self.lexer.input(),
            kind: error_kind,
        })
    }

    pub(crate) fn parse_replacement_exp(
        &mut self,
        declared_idents: Vec<&'source str>,
    ) -> Result<'source, ReplaceExpression<'source>> {
        let mut expressions = vec![];

        let mut token = self.token();

        use TokenKind::*;
        while token.kind != End {
            if let Lparen = token.kind {
                self.expect(Ident)?;
            }

            let exp = match &token.kind {
                Literal => AbstractReplaceExpression::Literal(&token.text),
                Ident => {
                    if !declared_idents.contains(&token.text) {
                        return Err(ParseError {
                            source: self.lexer.input(),
                            kind: ParseErrorKind::UndeclaredIdentifier {
                                ident: &token.text,
                                declared: declared_idents,
                                position: token.start,
                            },
                        });
                    }

                    AbstractReplaceExpression::Identifier(&token.text)
                }
                _ => {
                    token = self.token();
                    continue;
                }
            };

            expressions.push(exp);

            token = self.token();
        }

        Ok(ReplaceExpression {
            expressions: expressions.into(),
        })
    }

    pub fn parse(&mut self) -> Result<'source, MatchAndReplaceExpression<'source>> {
        let mex = self.parse_match_exp()?;
        let declared_idents = mex
            .expressions
            .iter()
            .filter_map(|e| match e {
                AbstractMatchingExpression::Literal(_) => None,
                AbstractMatchingExpression::Capture { identifier, .. } => Some(*identifier),
            })
            .collect();
        let expression = MatchAndReplaceExpression {
            rex: self.parse_replacement_exp(declared_idents)?,
            mex,
        };

        Ok(expression)
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_literal_expression() {
        let input = "abc";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression::new(vec![AbstractMatchingExpression::Literal("abc")])
        );

        let input = "1234";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression::new(vec![AbstractMatchingExpression::Literal("1234")],)
        )
    }

    #[test]
    fn test_capture_expression() {
        let input = "(num:int)";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression::new(vec![AbstractMatchingExpression::Capture {
                identifier: "num",
                identifier_type: CaptureType::Int
            }])
        );
    }

    #[test]
    fn test_simple_match_expression() {
        let input = "abc(d:dig)";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression::new(vec![
                AbstractMatchingExpression::Literal("abc"),
                AbstractMatchingExpression::Capture {
                    identifier: "d",
                    identifier_type: CaptureType::Digit
                }
            ])
        )
    }

    #[test]
    fn test_multiple_captures_in_match_expression() {
        let input = "abc235(d:dig)zap(num:int)(d:int)";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression::new(vec![
                AbstractMatchingExpression::Literal("abc235"),
                AbstractMatchingExpression::Capture {
                    identifier: "d",

                    identifier_type: CaptureType::Digit
                },
                AbstractMatchingExpression::Literal("zap"),
                AbstractMatchingExpression::Capture {
                    identifier: "num",

                    identifier_type: CaptureType::Int
                },
                AbstractMatchingExpression::Capture {
                    identifier: "d",

                    identifier_type: CaptureType::Int
                },
            ])
        )
    }

    #[test]
    fn test_wrong_capture_syntax() {
        let source = "(ident:)";
        let mut p = Parser::new(Lexer::new(source));
        assert_eq!(
            p.parse_match_exp().unwrap_err(),
            ParseError {
                source,
                kind: ParseErrorKind::ExpectedToken {
                    expected: TokenKind::Type,
                    found: TokenKind::Rparen,
                    text: ")",
                    position: 7
                }
            }
        );
    }

    #[test]
    fn test_simple_match_and_replace_expression() {
        let input = "(num:int)asdf->lul(num)";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression::new(vec![
                AbstractMatchingExpression::Capture {
                    identifier: "num",
                    identifier_type: CaptureType::Int
                },
                AbstractMatchingExpression::Literal("asdf"),
            ])
        );

        assert_eq!(
            p.parse_replacement_exp(vec!["num"]).unwrap(),
            ReplaceExpression {
                expressions: Box::new([
                    AbstractReplaceExpression::Literal("lul"),
                    AbstractReplaceExpression::Identifier("num")
                ])
            }
        )
    }
}
