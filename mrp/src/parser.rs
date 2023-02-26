use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

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
        identifier: Option<&'a str>,
        identifier_type: CaptureType,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum AbstractReplaceExpression<'a> {
    Literal(&'a str),
    Identifier(&'a str),
    CaptureIndex(usize),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum CaptureName<'a> {
    Ordinal(usize),
    Identifier(&'a str),
}

#[derive(Debug, PartialEq)]
pub struct MatchExpression<'a> {
    pub expressions: Vec<AbstractMatchingExpression<'a>>,
    captures: RefCell<HashMap<CaptureName<'a>, &'a str>>,
    curr_capture_index: Cell<usize>,
}

impl<'a> MatchExpression<'a> {
    pub fn new(expressions: Vec<AbstractMatchingExpression<'a>>) -> Self {
        Self {
            expressions,
            captures: RefCell::new(HashMap::new()),
            curr_capture_index: Cell::new(1),
        }
    }

    pub fn add_ordinal_capture(&self, value: &'a str) {
        self.captures
            .borrow_mut()
            .insert(CaptureName::Ordinal(self.curr_capture_index.get()), value);
        let bumped_index = self.curr_capture_index.get() + 1;
        self.curr_capture_index.set(bumped_index);
    }

    pub fn add_named_capture(&self, ident: &'a str, value: &'a str) {
        self.captures
            .borrow_mut()
            .insert(CaptureName::Identifier(ident), value);
    }

    pub fn get_capture_index(&self, index: usize) -> Option<&str> {
        self.captures
            .borrow()
            .get(&CaptureName::Ordinal(index))
            .map(|s| *s)
    }

    pub fn get_capture(&self, name: &str) -> Option<&str> {
        self.captures
            .borrow()
            .get(&CaptureName::Identifier(name))
            .map(|s| *s)
    }

    fn number_of_unamed_captures(&self) -> usize {
        self.expressions
            .iter()
            .filter_map(|e| match e {
                AbstractMatchingExpression::Literal(_) => None,
                AbstractMatchingExpression::Capture { identifier, .. } => match identifier {
                    Some(_) => None,
                    None => Some(()),
                },
            })
            .count()
    }

    fn declared_caputure_identifiers(&self) -> Vec<&'a str> {
        self.expressions
            .iter()
            .filter_map(|e| match e {
                AbstractMatchingExpression::Literal(_) => None,
                AbstractMatchingExpression::Capture { identifier, .. } => *identifier,
            })
            .collect()
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
                self.expect(&[Ident, Type])?;
            }

            let exp = match token.kind {
                Literal => AbstractMatchingExpression::Literal(&token.text),
                Type => {
                    let exp = self.parse_capture_indexed(token)?;
                    self.expect(&[Rparen])?;
                    exp
                }
                Ident => {
                    let exp = self.parse_capture_identifier(&token.text)?;
                    self.expect(&[Rparen])?;
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

    fn parse_capture_indexed(
        &mut self,
        token: Token<'a>,
    ) -> Result<'a, AbstractMatchingExpression<'a>> {
        Ok(AbstractMatchingExpression::Capture {
            identifier: None,
            identifier_type: match token {
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

    fn parse_capture_identifier(
        &mut self,
        identifier: &'a str,
    ) -> Result<'a, AbstractMatchingExpression<'a>> {
        self.eat_token();

        self.expect(&[TokenKind::Type])?;

        Ok(AbstractMatchingExpression::Capture {
            identifier: Some(identifier),
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

    fn expect(&mut self, token_kinds: &'static [TokenKind]) -> Result<'a, ()> {
        let error_kind = match self.peek_token() {
            t if token_kinds.contains(&t.kind) => return Ok(()),
            t => ParseErrorKind::ExpectedToken {
                expected: token_kinds,
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
        match_exp: &MatchExpression<'a>,
    ) -> Result<'a, ReplaceExpression<'a>> {
        let mut expressions = vec![];
        let declared_identifiers = match_exp.declared_caputure_identifiers();
        let number_unamed_captures = match_exp.number_of_unamed_captures();

        let mut token = self.token();

        use TokenKind::*;
        while token.kind != End {
            if let Lparen = token.kind {
                self.expect(&[Ident, CaptureIndex])?;
            }

            let exp = match &token.kind {
                Literal => AbstractReplaceExpression::Literal(&token.text),
                CaptureIndex => {
                    let idx_str = &token.text;
                    let idx = idx_str
                        .parse()
                        .expect("capture index should be a number for sure");

                    if number_unamed_captures < idx {
                        return Err(ParseError {
                            input: self.lexer.input(),
                            kind: ParseErrorKind::OutOfBoundsCaptureIndex {
                                index: idx_str,
                                number_of_ordinal_captures: number_unamed_captures,
                                position: token.start,
                            },
                        });
                    }

                    AbstractReplaceExpression::CaptureIndex(idx)
                }
                Ident => {
                    if !declared_identifiers.contains(&token.text) {
                        return Err(ParseError {
                            input: self.lexer.input(),
                            kind: ParseErrorKind::UndeclaredIdentifier {
                                ident: &token.text,
                                declared: declared_identifiers,
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
        let expression = MatchAndReplaceExpression {
            rex: self.parse_replacement_exp(&mex)?,
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
                identifier: Some("num"),
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
                    identifier: Some("d"),
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
                    identifier: Some("d"),

                    identifier_type: CaptureType::Digit
                },
                AbstractMatchingExpression::Literal("zap"),
                AbstractMatchingExpression::Capture {
                    identifier: Some("num"),

                    identifier_type: CaptureType::Int
                },
                AbstractMatchingExpression::Capture {
                    identifier: Some("d"),

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
                    expected: &[TokenKind::Type],
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

        let m_exp = MatchExpression::new(vec![
            AbstractMatchingExpression::Capture {
                identifier: Some("num"),
                identifier_type: CaptureType::Int,
            },
            AbstractMatchingExpression::Literal("asdf"),
        ]);

        assert_eq!(p.parse_match_exp().unwrap(), m_exp);

        assert_eq!(
            p.parse_replacement_exp(&m_exp).unwrap(),
            ReplaceExpression {
                expressions: vec![
                    AbstractReplaceExpression::Literal("lul"),
                    AbstractReplaceExpression::Identifier("num")
                ]
            }
        )
    }

    #[test]
    fn unnamed_positional_capture_groups() {
        let input = "(int)asdf(dig)->lul(1)(2)";
        let mut p = Parser::new(Lexer::new(input));

        let m_exp = MatchExpression::new(vec![
            AbstractMatchingExpression::Capture {
                identifier: None,
                identifier_type: CaptureType::Int,
            },
            AbstractMatchingExpression::Literal("asdf"),
            AbstractMatchingExpression::Capture {
                identifier: None,
                identifier_type: CaptureType::Digit,
            },
        ]);

        assert_eq!(p.parse_match_exp().unwrap(), m_exp);

        assert_eq!(
            p.parse_replacement_exp(&m_exp).unwrap(),
            ReplaceExpression {
                expressions: vec![
                    AbstractReplaceExpression::Literal("lul"),
                    AbstractReplaceExpression::CaptureIndex(1),
                    AbstractReplaceExpression::CaptureIndex(2)
                ]
            }
        )
    }
}
