use crate::lexer::{Lexer, Token};
use crate::parser::{self, Expression, MatchExpression, Parser};

fn match_on(pattern: MatchExpression, input: &str) -> bool {
    let mut curr_position = 0;
    let mut state = 0;
    let mut cap_start = None;
    let mut found_in_cap = None;

    while state < pattern.expressions.len() && curr_position < input.len() {
        let e = pattern.expressions.get(state).unwrap();
        match e {
            Expression::Literal(s) => {
                dbg!(s, input);
                if s.len() > input.len() {
                    return false;
                }

                let sub_str = &input[curr_position..s.len() + curr_position];

                let is_match = sub_str == *s;

                dbg!(is_match, sub_str);

                if is_match {
                    state += 1;
                    curr_position = s.len() + curr_position
                } else {
                    return false;
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

    return state == pattern.expressions.len();
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
fn five() {
    let exp = Parser::new(Lexer::new("ab(n:int)love(i:int)"))
        .parse_match_exp()
        .unwrap();
    assert_eq!(match_on(exp, "abb"), false);
}

#[test]
fn six() {
    let exp = Parser::new(Lexer::new("ab(n:int)love(i:int)"))
        .parse_match_exp()
        .unwrap();
    assert_eq!(match_on(exp, "ab321love78"), true);
}
