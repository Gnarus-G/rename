use crate::lexer::{Lexer, Token};
use crate::parser::{Expression, MatchExpression, Parser};

#[derive(Debug)]
pub(crate) struct Match<'t> {
    pub text: &'t str,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub struct MatchFinder<'t, 'p> {
    pub(crate) text: &'t str,
    pub(crate) pattern: &'p MatchExpression,
}

impl<'t, 'p> MatchFinder<'t, 'p> {
    pub fn new(pattern: &'p MatchExpression, text: &'t str) -> Self {
        Self { text, pattern }
    }

    pub(crate) fn find(&mut self) -> Option<Match<'t>> {
        if let Some((s, e)) = self.find_at(0) {
            return Some(Match {
                text: self.text,
                start: s,
                end: e,
            });
        }

        None
    }

    fn find_at(&mut self, start: usize) -> Option<(usize, usize)> {
        let pattern = self.pattern;
        let input = self.text;
        let mut curr_position = start;
        let mut state = 0;
        let mut cap_start = None;
        let mut found_in_cap = None;

        while state < pattern.expressions.len() && curr_position < input.len() {
            let e = pattern.expressions.get(state).unwrap();
            match e {
                Expression::Literal(literal) => {
                    dbg!(literal, input);
                    if literal.len() > input.len() {
                        return None;
                    }

                    let sub_str = &input[curr_position..literal.len() + curr_position];

                    let is_match = sub_str == *literal;

                    dbg!(is_match, sub_str);

                    if is_match {
                        state += 1;
                        curr_position = literal.len() + curr_position
                    } else {
                        return None;
                    }
                }
                Expression::Capture { identifier, typing } => match typing {
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
                            // e.set_value(input[cap_start.unwrap()..curr_position].to_string());
                        } else {
                            curr_position += 1;
                            state = 0;
                        }
                    }
                    _ => todo!(),
                },
                Expression::Identifier(_) => todo!(),
            }

            dbg!(e, curr_position, state, &input[..curr_position]);
        }

        if state == pattern.expressions.len() {
            return Some((start, curr_position));
        }

        None
    }
}

fn match_on(pattern: &MatchExpression, input: &str) -> bool {
    MatchFinder::new(pattern, input).find().is_some()
}

#[test]
fn one() {
    let exp = Parser::new(Lexer::new("abc")).parse_match_exp().unwrap();
    assert_eq!(match_on(&exp, "b"), false);
}

#[test]
fn two() {
    let exp = Parser::new(Lexer::new("ab")).parse_match_exp().unwrap();
    assert_eq!(match_on(&exp, "abc"), true);
}

#[test]
fn three() {
    let exp = Parser::new(Lexer::new("abc")).parse_match_exp().unwrap();
    assert_eq!(match_on(&exp, "abab5"), false);
}

#[test]
fn four() {
    let exp = Parser::new(Lexer::new("ab(n:int)"))
        .parse_match_exp()
        .unwrap();
    assert_eq!(match_on(&exp, "ab345"), true);
}

#[test]
fn five() {
    let exp = Parser::new(Lexer::new("ab(n:int)love(i:int)"))
        .parse_match_exp()
        .unwrap();
    assert_eq!(match_on(&exp, "abb"), false);
}

#[test]
fn six() {
    let exp = Parser::new(Lexer::new("ab(n:int)love(i:int)"))
        .parse_match_exp()
        .unwrap();
    assert_eq!(match_on(&exp, "ab321love78"), true);
}
