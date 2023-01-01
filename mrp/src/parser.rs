use std::{error::Error, str::FromStr};

use crate::lexer::{Lexer, Token};

type Result<'s, T> = std::result::Result<T, ParseError>;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    ExpectedToken { expected: Token, found: Token },
    UnsupportedToken(Token),
}
impl Error for ParseError {}
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

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    token: Token,
    peek_token: Token,
}

#[derive(Debug, PartialEq)]
enum Expression {
    Literal(String),
    Capture {
        identifier: Token,
        value: Option<String>,
        typing: Token,
    },
}

#[derive(Debug, PartialEq)]
struct MatchExpression {
    expressions: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct MatchAndReplaceExpression(MatchExpression);

impl FromStr for MatchAndReplaceExpression {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Parser::new(Lexer::new(s)).parse()
    }
}

impl<'l> Parser<'l> {
    fn new(lexer: Lexer<'l>) -> Self {
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

    fn parse(&mut self) -> Result<MatchAndReplaceExpression> {
        Ok(MatchAndReplaceExpression(self.parse_match_exp()?))
    }

    fn parse_match_exp(&mut self) -> Result<MatchExpression> {
        let mut expressions = vec![];

        while self.token != Token::Eof {
            let exp = match &self.token {
                Token::Literal(l) => self.parse_literal(l.clone()),
                Token::Ident(i) => self.parse_capture(i.clone())?,
                Token::Eof => todo!(),
                _ => {
                    self.advance();
                    continue;
                }
            };

            expressions.push(exp);

            self.advance();
        }

        Ok(MatchExpression { expressions })
    }

    fn parse_literal(&mut self, first_char: char) -> Expression {
        let mut lit = String::from(first_char);
        while let Token::Literal(ch) = self.peek_token {
            self.advance();
            lit.push(ch)
        }
        Expression::Literal(lit)
    }

    fn parse_capture(&mut self, identifier: String) -> Result<Expression> {
        self.advance();

        self.expect(Token::DigitType)
            .or(self.expect(Token::IntType))?;

        Ok(Expression::Capture {
            identifier: Token::Ident(identifier),
            value: None,
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
}

mod test {
    use super::*;

    #[test]
    fn test_literal_expression() {
        let input = "abc";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression {
                expressions: vec![Expression::Literal("abc".to_string())]
            }
        );

        let input = "1234";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression {
                expressions: vec![Expression::Literal("1234".to_string())]
            }
        )
    }

    #[test]
    fn test_capture_expression() {
        let input = "(num:int)";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression {
                expressions: vec![Expression::Capture {
                    identifier: Token::Ident("num".to_string()),
                    value: None,
                    typing: Token::IntType
                }]
            }
        );
    }

    #[test]
    fn test_simple_match_expression() {
        let input = "abc(d:dig)";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression {
                expressions: vec![
                    Expression::Literal("abc".to_string()),
                    Expression::Capture {
                        identifier: Token::Ident("d".to_string()),
                        value: None,
                        typing: Token::DigitType
                    }
                ]
            }
        )
    }

    #[test]
    fn test_multiple_captures_in_match_expression() {
        let input = "abc235(d:dig)zap(num:int)(d:int)";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression {
                expressions: vec![
                    Expression::Literal("abc235".to_string()),
                    Expression::Capture {
                        identifier: Token::Ident("d".to_string()),
                        value: None,
                        typing: Token::DigitType
                    },
                    Expression::Literal("zap".to_string()),
                    Expression::Capture {
                        identifier: Token::Ident("num".to_string()),
                        value: None,
                        typing: Token::IntType
                    },
                    Expression::Capture {
                        identifier: Token::Ident("d".to_string()),
                        value: None,
                        typing: Token::IntType
                    },
                ]
            }
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
}
