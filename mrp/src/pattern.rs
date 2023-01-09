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

pub struct Match<'t> {
    text: &'t str,
    pub start: usize,
    pub end: usize,
}

impl<'t> Match<'t> {
    fn as_str(&self) -> &str {
        &self.text[self.start..self.end]
    }
}

impl MatchExpression {
    fn find_at<'t>(&self, input: &'t str, start: usize) -> Option<Match<'t>> {
        let mut curr_position = start;
        let mut legit_start = start;
        let mut state = 0;
        let mut cap_start = None;
        let mut found_in_cap = None;

        let mut captures_map = self.captures.borrow_mut();

        while state < self.expressions.len() && curr_position < input.len() {
            let e = self.expressions.get(state).unwrap();

            match e {
                AbstractMatchingExpression::Literal(literal) => {
                    let slice_end = literal.len() + curr_position;
                    let slice_range = curr_position..slice_end;

                    let mut update_pointers = || {
                        curr_position = slice_end - 1;
                        legit_start = curr_position;
                    };

                    if slice_range.end > input.len() {
                        update_pointers();
                        continue;
                    }

                    let slice = &input[slice_range];

                    let is_match = slice == *literal;

                    if is_match {
                        state += 1;
                        curr_position = literal.len() + curr_position;
                    } else {
                        update_pointers();
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

                        let mut capture = |curr_position: usize| {
                            state += 1;
                            captures_map.insert(
                                identifier.to_string(),
                                input[cap_start.unwrap()..curr_position].to_string(),
                            );
                            cap_start = None;
                        };

                        if ch.is_ascii_digit() {
                            found_in_cap = Some(true);
                            curr_position += 1;

                            if curr_position == input.len() {
                                capture(curr_position);
                            }
                        } else if found_in_cap.is_some() {
                            capture(curr_position);
                        } else {
                            curr_position += 1;
                            state = 0;
                        }
                    }
                    t => panic!("{t} is an invalid capture type"),
                },
            }
        }

        if state == self.expressions.len() {
            return Some(Match {
                text: input,
                start: legit_start,
                end: curr_position,
            });
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
    type Item = Match<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_end >= self.text.len() {
            return None;
        }

        let m = match self.mex.find_at(self.text, self.last_end) {
            None => return None,
            Some(m) => m,
        };

        self.last_end = m.end;

        Some(m)
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
    let text = "ab321love78";

    assert_eq!(exp.find_at(text, 0).unwrap().as_str(), text);
    assert_eq!(exp.get_capture("n").unwrap(), "321");
    assert_eq!(exp.get_capture("i").unwrap(), "78");
}

#[test]
fn muliple_matches() {
    let pattern = MatchExpression::from_str("xy(n:int)").unwrap();
    let text = "wxy10xy33asdfxy81";
    let mut matches = Matches::new(&pattern, text);

    assert_eq!(matches.next().unwrap().as_str(), "xy10");
    assert_eq!(matches.next().unwrap().as_str(), "xy33");
    assert_eq!(matches.next().unwrap().as_str(), "xy81");
}
