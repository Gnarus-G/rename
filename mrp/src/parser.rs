use std::str::FromStr;

use crate::{
    error::{ParseError, Result},
    lexer::{Lexer, Token},
};

#[derive(Debug, PartialEq)]
enum Expression {
    Literal(String),
    Identifier(String),
    Capture { identifier: Token, typing: Token },
}

#[derive(Debug, PartialEq)]
struct MatchExpression {
    expressions: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
struct ReplaceExpression {
    expressions: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct MatchAndReplaceExpression(MatchExpression, ReplaceExpression);

impl FromStr for MatchAndReplaceExpression {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Parser::new(Lexer::new(s)).parse()
    }
}

impl MatchAndReplaceExpression {
    pub fn apply(&self, strings: Vec<&str>) -> Vec<String> {
        let MatchAndReplaceExpression(match_exp, replace_exp) = &self;
        let regex_pattern: String = match_exp
            .expressions
            .iter()
            .filter_map(|e| match e {
                Expression::Literal(l) => Some(l.clone()),
                Expression::Capture { identifier, typing } => {
                    if let Token::Ident(id) = identifier {
                        return match typing {
                            Token::DigitType => Some(format!("(?P<{id}>\\d)")),
                            Token::IntType => Some(format!("(?P<{id}>\\d+)")),
                            _ => None,
                        };
                    };

                    None
                }
                Expression::Identifier(_) => None,
            })
            .collect();

        let regex_replacement: String = replace_exp
            .expressions
            .iter()
            .filter_map(|e| match e {
                Expression::Literal(l) => Some(l.clone()),
                Expression::Capture {
                    identifier: _,
                    typing: _,
                } => None,
                Expression::Identifier(id) => Some(format!("${{{id}}}")),
            })
            .collect();

        let pattern = regex::Regex::new(&regex_pattern).unwrap();

        return strings
            .iter()
            .map(|s| pattern.replace(s, &regex_replacement).to_string())
            .collect();
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    token: Token,
    peek_token: Token,
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
        Ok(MatchAndReplaceExpression(
            self.parse_match_exp()?,
            self.parse_replacement_exp()?,
        ))
    }

    fn parse_match_exp(&mut self) -> Result<MatchExpression> {
        let mut expressions = vec![];

        while self.token != Token::Eof {
            let exp = match &self.token {
                Token::Literal(l) => self.parse_literal(l.clone()),
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

    fn parse_replacement_exp(&mut self) -> Result<ReplaceExpression> {
        let mut expressions = vec![];

        while self.token != Token::Eof {
            let exp = match &self.token {
                Token::Literal(l) => self.parse_literal(l.clone()),
                Token::Ident(i) => Expression::Identifier(i.clone()),
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

                        typing: Token::DigitType
                    },
                    Expression::Literal("zap".to_string()),
                    Expression::Capture {
                        identifier: Token::Ident("num".to_string()),

                        typing: Token::IntType
                    },
                    Expression::Capture {
                        identifier: Token::Ident("d".to_string()),

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

    #[test]
    fn test_simple_match_and_replace_expression() {
        let input = "(num:int)asdf->lul(num)";
        let mut p = Parser::new(Lexer::new(input));

        assert_eq!(
            p.parse_match_exp().unwrap(),
            MatchExpression {
                expressions: vec![
                    Expression::Capture {
                        identifier: Token::Ident("num".to_string()),

                        typing: Token::IntType
                    },
                    Expression::Literal("asdf".to_string()),
                ]
            }
        );

        assert_eq!(
            p.parse_replacement_exp().unwrap(),
            ReplaceExpression {
                expressions: vec![
                    Expression::Literal("lul".to_string()),
                    Expression::Identifier("num".to_string())
                ]
            }
        )
    }

    #[test]
    fn test_mrp_application() {
        let input = "(num:int)asdf->lul(num)";
        let expression = MatchAndReplaceExpression::from_str(input).unwrap();

        let treated = expression.apply(vec!["124asdf", "3asdfwery", "lk234asdfas"]);

        assert_eq!(treated, vec!["lul124", "lul3wery", "lklul234as"]);

        let expression = MatchAndReplaceExpression::from_str("hello(as:dig)->oh(as)hi").unwrap();

        let treated = expression.apply(vec!["hello5", "ashello090", "hello345hello"]);

        assert_eq!(treated, vec!["oh5hi", "asoh0hi90", "oh3hi45hello"]);
    }
}
