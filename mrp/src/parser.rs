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
pub enum AbstractMatchingExpression {
    Literal(String),
    Capture {
        identifier: String,
        identifier_type: CaptureType,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum AbstractReplaceExpression {
    Literal(String),
    Identifier(String),
}

#[derive(Debug, PartialEq)]
pub struct MatchExpression {
    pub expressions: Vec<AbstractMatchingExpression>,
    pub captures: RefCell<HashMap<String, String>>,
}

impl MatchExpression {
    pub fn new(expressions: Vec<AbstractMatchingExpression>) -> Self {
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
pub struct ReplaceExpression {
    pub expressions: Vec<AbstractReplaceExpression>,
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    token: Token<'a>,
    peek_token: Token<'a>,
}

impl<'a> Parser<'a> {
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

    pub fn parse_match_exp(&mut self) -> Result<'a, MatchExpression> {
        let mut expressions = vec![];

        while self.token.kind != TokenKind::End {
            let exp = match self.token.kind {
                TokenKind::Literal => {
                    AbstractMatchingExpression::Literal(self.token.text.to_string())
                }
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

    fn parse_capture(&mut self) -> Result<'a, AbstractMatchingExpression> {
        let identifier = self.token.clone();
        self.advance();

        self.expect(TokenKind::DigitType)
            .or(self.expect(TokenKind::IntType))?;

        Ok(AbstractMatchingExpression::Capture {
            identifier: identifier.text.to_string(),
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

    pub fn parse_replacement_exp(&mut self) -> Result<ReplaceExpression> {
        let mut expressions = vec![];

        while self.token.kind != TokenKind::End {
            let exp = match &self.token.kind {
                TokenKind::Literal => {
                    AbstractReplaceExpression::Literal(self.token.text.to_string())
                }
                TokenKind::Ident => {
                    AbstractReplaceExpression::Identifier(self.token.text.to_string())
                }
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
            MatchExpression::new(vec![AbstractMatchingExpression::Literal("abc".to_string())])
        );

        let input = "1234";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression::new(vec![AbstractMatchingExpression::Literal(
                "1234".to_string()
            )],)
        )
    }

    #[test]
    fn test_capture_expression() {
        let input = "(num:int)";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression::new(vec![AbstractMatchingExpression::Capture {
                identifier: "num".to_string(),
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
                AbstractMatchingExpression::Literal("abc".to_string()),
                AbstractMatchingExpression::Capture {
                    identifier: "d".to_string(),
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
                AbstractMatchingExpression::Literal("abc235".to_string()),
                AbstractMatchingExpression::Capture {
                    identifier: ("d".to_string()),

                    identifier_type: CaptureType::Digit
                },
                AbstractMatchingExpression::Literal("zap".to_string()),
                AbstractMatchingExpression::Capture {
                    identifier: ("num".to_string()),

                    identifier_type: CaptureType::Int
                },
                AbstractMatchingExpression::Capture {
                    identifier: ("d".to_string()),

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
                    identifier: "num".to_string(),
                    identifier_type: CaptureType::Int
                },
                AbstractMatchingExpression::Literal("asdf".to_string()),
            ])
        );

        assert_eq!(
            p.parse_replacement_exp().unwrap(),
            ReplaceExpression {
                expressions: vec![
                    AbstractReplaceExpression::Literal("lul".to_string()),
                    AbstractReplaceExpression::Identifier("num".to_string())
                ]
            }
        )
    }
}
