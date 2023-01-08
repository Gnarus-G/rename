use std::str::FromStr;

use crate::error::ParseError;
use crate::lexer::{Lexer, Token};
use crate::parser::{AbstractMatchingExpression, MatchExpression, Parser};

#[cfg(test)]
impl FromStr for MatchExpression {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Parser::new(Lexer::new(s)).parse_match_exp()
    }
}

impl MatchExpression {
    fn find_at(&self, input: &str, start: usize) -> Option<(usize, usize)> {
        let mut curr_position = start;
        let mut legit_start = start;
        let mut state = 0;
        let mut cap_start = None;
        let mut found_in_cap = None;

        let mut captures_map = self.captures.borrow_mut();

        while state < self.expressions.len() && curr_position < input.len() {
            let e = self.expressions.get(state).unwrap();

            dbg!(curr_position, state);

            match e {
                AbstractMatchingExpression::Literal(literal) => {
                    let lit_end_in_input = literal.len() + curr_position;
                    let lit_range = curr_position..lit_end_in_input;

                    if lit_range.end > input.len() {
                        curr_position = lit_end_in_input - 1;
                        legit_start = curr_position;
                        continue;
                    }

                    dbg!(literal, &lit_range);

                    let sub_str = &input[lit_range];

                    let is_match = sub_str == *literal;

                    dbg!(is_match, sub_str);

                    if is_match {
                        state += 1;
                        curr_position = literal.len() + curr_position;
                    } else {
                        curr_position = lit_end_in_input - 1;
                        legit_start = curr_position;
                        continue;
                    }
                }
                AbstractMatchingExpression::Capture { identifier, typing } => match typing {
                    Token::DigitType => todo!(),
                    Token::IntType => {
                        let ch = input.as_bytes()[curr_position] as char;
                        if let None = cap_start {
                            cap_start = Some(curr_position);
                        }

                        if ch.is_ascii_digit() {
                            found_in_cap = Some(true);
                            curr_position += 1;

                            if curr_position == input.len() {
                                state += 1;
                            }
                        } else if found_in_cap.is_some() {
                            // is a match
                            state += 1;
                            captures_map.insert(
                                identifier.to_string(),
                                input[cap_start.unwrap()..curr_position].to_string(),
                            );
                        } else {
                            curr_position += 1;
                            state = 0;
                        }
                    }
                    _ => todo!(),
                },
            }
        }

        if state == self.expressions.len() {
            return Some((legit_start, curr_position));
        }

        None
    }
    pub fn find_iter<'m, 't>(&'m self, text: &'t str) -> Matches<'t, 'm> {
        Matches::new(self, text)
    }
}

#[derive(Debug)]
pub struct Matches<'t, 'm> {
    pub(crate) text: &'t str,
    pub(crate) mex: &'m MatchExpression,
    last_end: usize,
}

impl<'t, 'm> Matches<'t, 'm> {
    pub fn new(mex: &'m MatchExpression, text: &'t str) -> Self {
        Self {
            text,
            mex,
            last_end: 0,
        }
    }
}

impl<'t, 'm> Iterator for Matches<'t, 'm> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_end >= self.text.len() {
            return None;
        }

        let (s, e) = match self.mex.find_at(self.text, self.last_end) {
            None => return None,
            Some((s, e)) => (s, e),
        };

        self.last_end = e;

        Some((s, e))
    }
}

#[cfg(test)]
fn match_on(pattern: MatchExpression, input: &str) -> bool {
    Matches::new(&pattern, input).count() > 0
}

#[test]
fn one() {
    let exp = Parser::new(Lexer::new("abc")).parse_match_exp().unwrap();
    assert_eq!(match_on(exp, "b"), false);
}

#[test]
fn two() {
    let exp = Parser::new(Lexer::new("ab")).parse_match_exp().unwrap();
    assert_eq!(match_on(exp, "abc"), true);
}

#[test]
fn three() {
    let exp = Parser::new(Lexer::new("abc")).parse_match_exp().unwrap();
    assert_eq!(match_on(exp, "abab5"), false);
}

#[test]
fn four() {
    let exp = Parser::new(Lexer::new("ab(n:int)"))
        .parse_match_exp()
        .unwrap();
    assert_eq!(match_on(exp, "ab345"), true);
}

#[test]
fn sub_str_at_the_end() {
    let exp = Parser::new(Lexer::new("ab(n:int)"))
        .parse_match_exp()
        .unwrap();
    assert_eq!(match_on(exp, "helloab345"), true);
}

#[test]
fn five() {
    let exp = Parser::new(Lexer::new("ab(n:int)love(i:int)"))
        .parse_match_exp()
        .unwrap();
    assert_eq!(match_on(exp, "abb"), false);
}

#[test]
fn two_capture_groups() {
    let exp = Parser::new(Lexer::new("ab(n:int)love(i:int)"))
        .parse_match_exp()
        .unwrap();

    assert_eq!(match_on(exp, "ab321love78"), true);
}

#[test]
fn muliple_matches() {
    let pattern = MatchExpression::from_str("xy(n:int)").unwrap();
    let text = "wxy10xy33asdfxy81";
    let mut matches = Matches::new(&pattern, text);

    assert_eq!(matches.next(), Some((1, 5)));
    assert_eq!(matches.next(), Some((5, 9)));
    assert_eq!(matches.next(), Some((13, text.len())));
}
