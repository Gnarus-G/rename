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

        while token.kind != TokenKind::End {
            let exp = match token.kind {
                TokenKind::Literal => AbstractMatchingExpression::Literal(&token.text),
                TokenKind::Ident => self.parse_capture(&token.text)?,
                TokenKind::Arrow => break,
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

        self.expect(TokenKind::DigitType)
            .or(self.expect(TokenKind::IntType))?;

        Ok(AbstractMatchingExpression::Capture {
            identifier,
            identifier_type: match self.token().kind {
                TokenKind::DigitType => CaptureType::Digit,
                TokenKind::IntType => CaptureType::Int,
                _ => unreachable!(),
            },
        })
    }

    fn expect(&mut self, token_kind: TokenKind) -> Result<'a, ()> {
        match self.peek_token() {
            t if t.kind == token_kind => Ok(()),
            t => {
                return Err(ParseError::ExpectedToken {
                    expected: token_kind,
                    found: t.clone(),
                });
            }
        }
    }

    pub(crate) fn parse_replacement_exp(&mut self) -> Result<'a, ReplaceExpression<'a>> {
        let mut expressions = vec![];

        let mut token = self.token();
        while token.kind != TokenKind::End {
            let exp = match &token.kind {
                TokenKind::Literal => AbstractReplaceExpression::Literal(&token.text),
                TokenKind::Ident => AbstractReplaceExpression::Identifier(&token.text),
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
