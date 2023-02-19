use std::{cell::RefCell, collections::HashMap};

use crate::{
    error::{ParseError, ParseErrorKind, Result},
    lexer::{Lexer, Token, TokenKind},
};

#[derive(Debug, PartialEq)]
pub enum CaptureType {
    Int,
    Digit,
}

#[derive(Debug, PartialEq)]
pub enum AbstractMatchingExpression<'a> {
    Literal(&'a str),
    Capture {
        identifier: &'a str,
        identifier_type: CaptureType,
    },
}

impl<'a> AbstractMatchingExpression<'a> {
    fn is_capture(&self) -> bool {
        match self {
            AbstractMatchingExpression::Literal(_) => false,
            AbstractMatchingExpression::Capture { .. } => true,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AbstractReplaceExpression<'a> {
    Literal(&'a str),
    Identifier(&'a str),
}

#[derive(Debug, PartialEq)]
pub struct MatchExpression<'a> {
    pub expressions: Vec<AbstractMatchingExpression<'a>>,
    pub captures: RefCell<HashMap<&'a str, &'a str>>,
}

impl<'a> MatchExpression<'a> {
    pub fn new(expressions: Vec<AbstractMatchingExpression<'a>>) -> Self {
        Self {
            expressions,
            captures: RefCell::new(HashMap::new()),
        }
    }

    pub fn get_capture(&self, name: &str) -> Option<&str> {
        self.captures.borrow().get(name).map(|s| *s)
    }
}

#[derive(Debug, PartialEq)]
pub struct ReplaceExpression<'a> {
    pub expressions: Vec<AbstractReplaceExpression<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct MatchAndReplaceExpression<'a> {
    pub mex: MatchExpression<'a>,
    pub rex: ReplaceExpression<'a>,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    peeked: Option<Token<'a>>,
}

impl<'a> From<&'a str> for Parser<'a> {
    fn from(input: &'a str) -> Self {
        Self::new(Lexer::new(input))
    }
}

impl<'a> From<&'a String> for Parser<'a> {
    fn from(input: &'a String) -> Self {
        Self::new(Lexer::new(&input))
    }
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            peeked: None,
        }
    }

    fn token(&mut self) -> Token<'a> {
        match self.peeked.take() {
            Some(t) => t,
            None => self.lexer.next_token(),
        }
    }

    fn peek_token(&mut self) -> &Token<'a> {
        self.peeked.get_or_insert_with(|| self.lexer.next_token())
    }

    fn eat_token(&mut self) {
        self.token();
    }

    pub(crate) fn parse_match_exp(&mut self) -> Result<'a, MatchExpression<'a>> {
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

    fn parse_capture(&mut self, identifier: &'a str) -> Result<'a, AbstractMatchingExpression<'a>> {
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
                            input: self.lexer.input(),
                            kind: ParseErrorKind::UnsupportedToken(t),
                        })
                    }
                },
                _ => unreachable!("we expected a type token"),
            },
        })
    }

    fn expect(&mut self, token_kind: TokenKind) -> Result<'a, ()> {
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
            input: self.lexer.input(),
            kind: error_kind,
        })
    }

    fn expect_not(&mut self, token_kind: TokenKind, current: TokenKind) -> Result<'a, ()> {
        let error_kind = match self.peek_token() {
            t if t.kind == token_kind => ParseErrorKind::UnexpectedToken {
                unexpected: token_kind,
                previous: current,
                position: t.start,
            },
            _ => return Ok(()),
        };

        Err(ParseError {
            input: self.lexer.input(),
            kind: error_kind,
        })
    }

    pub(crate) fn parse_replacement_exp(
        &mut self,
        declared_idents: Vec<&'a str>,
    ) -> Result<'a, ReplaceExpression<'a>> {
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
                            input: self.lexer.input(),
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

        Ok(ReplaceExpression { expressions })
    }

    pub fn parse(&mut self) -> Result<'a, MatchAndReplaceExpression<'a>> {
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
        let input = "(ident:)";
        let mut p = Parser::new(Lexer::new(input));
        assert_eq!(
            p.parse_match_exp().unwrap_err(),
            ParseError {
                input,
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
                expressions: vec![
                    AbstractReplaceExpression::Literal("lul"),
                    AbstractReplaceExpression::Identifier("num")
                ]
            }
        )
    }
}
