use std::{cell::RefCell, collections::HashMap};

use crate::{
    error::{ParseError, Result},
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

#[derive(Debug, PartialEq, Clone)]
pub enum AbstractReplaceExpression<'a> {
    Literal(&'a str),
    Identifier(&'a str),
}

#[derive(Debug, PartialEq)]
pub struct MatchExpression<'a> {
    pub expressions: Vec<AbstractMatchingExpression<'a>>,
    pub captures: RefCell<HashMap<&'a str, String>>,
}

impl<'a> MatchExpression<'a> {
    pub fn new(expressions: Vec<AbstractMatchingExpression<'a>>) -> Self {
        Self {
            expressions,
            captures: RefCell::new(HashMap::new()),
        }
    }

    pub fn get_capture(&self, name: &str) -> Option<String> {
        self.captures.borrow().get(name).map(|s| s.to_string())
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
    token: Token<'a>,
    peek_token: Token<'a>,
}

impl<'a> Parser<'a> {
    pub fn from_input(input: &'a str) -> Self {
        Self::new(Lexer::new(input))
    }

    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut p = Self {
            lexer,
            token: Token {
                kind: TokenKind::End,
                text: crate::lexer::TokenText::Empty,
                start: 0,
            },
            peek_token: Token {
                kind: TokenKind::End,
                text: crate::lexer::TokenText::Empty,
                start: 0,
            },
        };
        p.advance();
        p.advance();
        p
    }

    fn advance(&mut self) {
        self.token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    pub(crate) fn parse_match_exp(&mut self) -> Result<'a, MatchExpression<'a>> {
        let mut expressions = vec![];

        while self.token.kind != TokenKind::End {
            let exp = match self.token.kind {
                TokenKind::Literal => AbstractMatchingExpression::Literal(&self.token.text),
                TokenKind::Ident => self.parse_capture()?,
                TokenKind::Arrow => {
                    self.advance();
                    break;
                }
                _ => {
                    self.advance();
                    continue;
                }
            };

            expressions.push(exp);

            self.advance();
        }

        Ok(MatchExpression::new(expressions))
    }

    fn parse_capture(&mut self) -> Result<'a, AbstractMatchingExpression<'a>> {
        let identifier = self.token.clone();
        self.advance();

        self.expect(TokenKind::DigitType)
            .or(self.expect(TokenKind::IntType))?;

        Ok(AbstractMatchingExpression::Capture {
            identifier: &identifier.text,
            identifier_type: match self.token.kind {
                TokenKind::DigitType => CaptureType::Digit,
                TokenKind::IntType => CaptureType::Int,
                _ => unreachable!(),
            },
        })
    }

    fn expect(&mut self, token_kind: TokenKind) -> Result<'a, ()> {
        if self.peek_token.kind != token_kind {
            return Err(ParseError::ExpectedToken {
                expected: token_kind,
                found: self.peek_token.clone(),
            });
        }

        self.advance();
        Ok(())
    }

    pub(crate) fn parse_replacement_exp(&mut self) -> Result<'a, ReplaceExpression<'a>> {
        let mut expressions = vec![];

        while self.token.kind != TokenKind::End {
            let exp = match &self.token.kind {
                TokenKind::Literal => AbstractReplaceExpression::Literal(&self.token.text),
                TokenKind::Ident => AbstractReplaceExpression::Identifier(&self.token.text),
                _ => {
                    self.advance();
                    continue;
                }
            };

            expressions.push(exp);

            self.advance();
        }

        Ok(ReplaceExpression { expressions })
    }

    pub fn parse(&mut self) -> Result<'a, MatchAndReplaceExpression<'a>> {
        let expression = MatchAndReplaceExpression {
            mex: self.parse_match_exp()?,
            rex: self.parse_replacement_exp()?,
        };

        Ok(expression)
    }
}

#[cfg(test)]
mod tests {

    use crate::lexer::TokenText;

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
            ParseError::ExpectedToken {
                expected: TokenKind::IntType,
                found: Token {
                    kind: TokenKind::Rparen,
                    text: TokenText::Empty,
                    start: 7
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
            p.parse_replacement_exp().unwrap(),
            ReplaceExpression {
                expressions: vec![
                    AbstractReplaceExpression::Literal("lul"),
                    AbstractReplaceExpression::Identifier("num")
                ]
            }
        )
    }
}
