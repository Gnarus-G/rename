use std::{cell::RefCell, collections::HashMap};

use crate::{
    error::{ParseError, Result},
    lexer::{Lexer, Token},
};

#[derive(Debug, PartialEq)]
pub enum AbstractMatchingExpression {
    Literal(String),
    Capture { identifier: Token, typing: Token },
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

pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    token: Token,
    peek_token: Token,
}

impl<'l> Parser<'l> {
    pub fn new(lexer: Lexer<'l>) -> Self {
        let mut p = Self {
            lexer,
            token: Token::Eof,
            peek_token: Token::Eof,
        };
        p.advance();
        p.advance();
        p
    }

    fn advance(&mut self) {
        self.token = self.peek_token.clone();
        self.peek_token = self.lexer.next();
    }

    pub fn parse_match_exp(&mut self) -> Result<MatchExpression> {
        let mut expressions = vec![];

        while self.token != Token::Eof {
            let exp = match &self.token {
                Token::Literal(l) => {
                    AbstractMatchingExpression::Literal(self.parse_literal(l.clone()))
                }
                Token::Ident(i) => self.parse_capture(i.clone())?,
                Token::Arrow => {
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

    fn parse_literal(&mut self, first_char: char) -> String {
        let mut lit = String::from(first_char);
        while let Token::Literal(ch) = self.peek_token {
            self.advance();
            lit.push(ch)
        }
        lit
    }

    fn parse_capture(&mut self, identifier: String) -> Result<AbstractMatchingExpression> {
        self.advance();

        self.expect(Token::DigitType)
            .or(self.expect(Token::IntType))?;

        Ok(AbstractMatchingExpression::Capture {
            identifier: Token::Ident(identifier),

            typing: self.token.clone(),
        })
    }

    fn expect(&mut self, token: Token) -> Result<()> {
        if self.peek_token != token {
            return Err(ParseError::ExpectedToken {
                expected: token,
                found: self.peek_token.clone(),
            });
        }

        self.advance();
        Ok(())
    }

    pub fn parse_replacement_exp(&mut self) -> Result<ReplaceExpression> {
        let mut expressions = vec![];

        while self.token != Token::Eof {
            let exp = match &self.token {
                Token::Literal(l) => {
                    AbstractReplaceExpression::Literal(self.parse_literal(l.clone()))
                }
                Token::Ident(i) => AbstractReplaceExpression::Identifier(i.clone()),
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
                identifier: Token::Ident("num".to_string()),

                typing: Token::IntType
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
                    identifier: Token::Ident("d".to_string()),

                    typing: Token::DigitType
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
                    identifier: Token::Ident("d".to_string()),

                    typing: Token::DigitType
                },
                AbstractMatchingExpression::Literal("zap".to_string()),
                AbstractMatchingExpression::Capture {
                    identifier: Token::Ident("num".to_string()),

                    typing: Token::IntType
                },
                AbstractMatchingExpression::Capture {
                    identifier: Token::Ident("d".to_string()),

                    typing: Token::IntType
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
                expected: Token::IntType,
                found: Token::Rparen
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
                    identifier: Token::Ident("num".to_string()),

                    typing: Token::IntType
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
