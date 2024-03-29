use crate::{
    captures::Captures,
    parser::{AbstractMatchingExpression, CaptureType, MatchExpression},
};

pub struct Match<'input> {
    input: &'input str,
    pub start: usize,
    pub end: usize,
}

impl<'input> Match<'input> {
    pub fn as_str(&self) -> &str {
        &self.input[self.start..self.end]
    }
}

impl<'source> MatchExpression<'source> {
    pub fn find_at_capturing<'input>(
        &self,
        input: &'input str,
        start: usize,
    ) -> (Option<Match<'input>>, Captures<'source, 'input>) {
        let mut curr_position = start;
        let mut legit_start = start;
        let mut state = 0;
        let mut capture_slice_start = None;
        let mut capture_candidate_found = None;
        let input_bytes = input.as_bytes();

        let mut captures = Captures::new();

        while state < self.expressions.len() && curr_position < input_bytes.len() {
            let e = self.get_expression(state).unwrap();

            match e {
                AbstractMatchingExpression::Literal(literal) => {
                    let slice_end = literal.len() + curr_position;
                    let slice_range = curr_position..slice_end;

                    let mut update_pointers = || {
                        curr_position += 1;
                        legit_start = curr_position;
                    };

                    if slice_range.end > input_bytes.len() {
                        update_pointers();
                        continue;
                    }

                    let slice = &input_bytes[slice_range];

                    let is_match = slice == literal.as_bytes();

                    if is_match {
                        state += 1;
                        curr_position += literal.len();
                    } else {
                        update_pointers();
                        continue;
                    }
                }
                AbstractMatchingExpression::Capture {
                    identifier,
                    identifier_type,
                } => match identifier_type {
                    CaptureType::Digit => {
                        let ch = input_bytes[curr_position];
                        let ch_str = &input_bytes[curr_position..curr_position + 1];

                        if ch.is_ascii_digit() {
                            curr_position += 1;
                            state += 1;
                            let captured_digit = &std::str::from_utf8(ch_str).unwrap();
                            captures.put(identifier, captured_digit);
                        } else {
                            curr_position += 1;
                            state = 0;
                        }
                    }
                    CaptureType::Int => {
                        let ch = input_bytes[curr_position] as char;

                        let mut capture = |start: usize, curr_position: usize| {
                            let captured_int =
                                &std::str::from_utf8(&input_bytes[start..curr_position]).unwrap();
                            captures.put(identifier, captured_int);
                        };

                        if ch.is_ascii_digit() {
                            if capture_slice_start.is_none() {
                                capture_slice_start = Some(curr_position);
                                if state == 0 {
                                    legit_start = curr_position;
                                }
                            }

                            capture_candidate_found = Some(true);
                            curr_position += 1;

                            if curr_position == input_bytes.len() {
                                state += 1;
                                capture(capture_slice_start.unwrap(), curr_position);
                                capture_slice_start = None;
                            }
                        } else if capture_candidate_found.is_some() {
                            state += 1;
                            capture(capture_slice_start.unwrap(), curr_position);
                            capture_slice_start = None;
                        } else {
                            curr_position += 1;
                            state = 0;
                        }
                    }
                },
            }
        }

        if state == self.expressions.len() {
            return (
                Some(Match {
                    input,
                    start: legit_start,
                    end: curr_position,
                }),
                captures,
            );
        }

        (None, captures)
    }

    /// Find the leftmost-first match in the input starting at the given position
    pub fn find_at<'input>(&self, input: &'input str, start: usize) -> Option<Match<'input>> {
        self.find_at_capturing(input, start).0
    }

    pub fn find_iter<'input>(self, input: &'input str) -> Matches<'input, 'source> {
        Matches::new(self, input)
    }
}

#[derive(Debug)]
pub struct Matches<'input, 'source> {
    pub(crate) input: &'input str,
    pub(crate) mex: MatchExpression<'source>,
    last_end: usize,
}

impl<'input, 'source> Matches<'input, 'source> {
    pub fn new(mex: MatchExpression<'source>, input: &'input str) -> Self {
        Self {
            input,
            mex,
            last_end: 0,
        }
    }
}

impl<'input, 'source> Iterator for Matches<'input, 'source> {
    type Item = Match<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_end >= self.input.len() {
            return None;
        }

        let m = match self.mex.find_at(self.input, self.last_end) {
            None => return None,
            Some(m) => m,
        };

        self.last_end = m.end;

        Some(m)
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    #[test]
    fn match_counts() {
        macro_rules! assert_match_on {
            ($pattern:literal, $input:literal) => {
                let exp = Parser::new(Lexer::new($pattern)).parse_match_exp().unwrap();
                assert!(Matches::new(exp, $input).count() > 0);
            };
            ($pattern:literal, $input:literal, $boolean:literal) => {
                let exp = Parser::new(Lexer::new($pattern)).parse_match_exp().unwrap();
                assert_eq!(Matches::new(exp, $input).count() > 0, $boolean);
            };
        }

        assert_match_on!("abc", "b", false);
        assert_match_on!("ab", "abc");
        assert_match_on!("abc", "abab5", false);
        assert_match_on!("ab(n:int)", "ab345");
        assert_match_on!("ab(n:int)", "helloab345");
        assert_match_on!("ab(n:int)love(i:int)", "abb", false);
    }

    #[test]
    fn two_capture_groups() {
        let exp = Parser::new(Lexer::new("ab(n:int)love(i:int)"))
            .parse_match_exp()
            .unwrap();
        let text = "ab321love78";

        assert_eq!(exp.find_at(text, 0).unwrap().as_str(), text);

        let cap = exp.find_at_capturing(text, 0).1;

        assert_eq!(cap.get("n").unwrap(), "321");
        assert_eq!(cap.get("i").unwrap(), "78");
    }

    #[test]
    fn digit_capture_group() {
        let exp = Parser::new(Lexer::new("digit(d:dig)"))
            .parse_match_exp()
            .unwrap();
        let text = "aewrdigit276yoypa";

        assert_eq!(exp.find_at(text, 0).unwrap().as_str(), "digit2");
        let cap = exp.find_at_capturing(text, 0).1;
        assert_eq!(cap.get("d").unwrap(), "2");
    }

    #[test]
    fn three_capture_groups() {
        let exp = Parser::new(Lexer::new("ab(n:int)love(i:int)ly(d:dig)"))
            .parse_match_exp()
            .unwrap();
        let text = "ab321love78ly8";

        assert_eq!(exp.find_at(text, 0).unwrap().as_str(), text);
        let cap = exp.find_at_capturing(text, 0).1;
        assert_eq!(cap.get("n").unwrap(), "321");
        assert_eq!(cap.get("i").unwrap(), "78");
        assert_eq!(cap.get("d").unwrap(), "8");
    }

    #[test]
    fn int_capture_group_at_the_begining() {
        let exp = Parser::new(Lexer::new("(n:int)love(i:int)ly(d:dig)"))
            .parse_match_exp()
            .unwrap();
        let text = "ab321love78ly8";

        assert_eq!(exp.find_at(text, 0).unwrap().as_str(), &text[2..]);

        let cap = exp.find_at_capturing(text, 0).1;
        assert_eq!(cap.get("n").unwrap(), "321");
        assert_eq!(cap.get("i").unwrap(), "78");
        assert_eq!(cap.get("d").unwrap(), "8");
    }

    #[test]
    fn special() {
        let exp = MatchExpression::from_str("hello(as:dig)->oh(as)hi").unwrap();
        assert_eq!(exp.find_at("ashello090", 0).unwrap().as_str(), "hello0");
    }

    #[test]
    fn muliple_matches() {
        let pattern = MatchExpression::from_str("xy(n:int)").unwrap();
        let text = "wxy10xy33asdfxy81";
        let mut matches = Matches::new(pattern, text);

        assert_eq!(matches.next().unwrap().as_str(), "xy10");
        assert_eq!(matches.next().unwrap().as_str(), "xy33");
        assert_eq!(matches.next().unwrap().as_str(), "xy81");
    }
}
