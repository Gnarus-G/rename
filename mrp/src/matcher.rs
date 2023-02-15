use crate::parser::{AbstractMatchingExpression, CaptureType, MatchExpression};

pub struct Match<'t> {
    text: &'t str,
    pub start: usize,
    pub end: usize,
}

impl<'t> Match<'t> {
    pub fn as_str(&self) -> &str {
        &self.text[self.start..self.end]
    }
}

impl<'a> MatchExpression<'a> {
    /// Find the leftmost-first match in the input starting at the given position
    pub fn find_at<'t: 'a, 's: 'a>(&'s self, input: &'t str, start: usize) -> Option<Match<'t>> {
        let mut curr_position = start;
        let mut legit_start = start;
        let mut state = 0;
        let mut capture_slice_start = None;
        let mut capture_candidate_found = None;

        let mut captures_map = self.captures.borrow_mut();

        while state < self.expressions.len() && curr_position < input.len() {
            let e = self.expressions.get(state).unwrap();

            match e {
                AbstractMatchingExpression::Literal(literal) => {
                    let slice_end = literal.len() + curr_position;
                    let slice_range = curr_position..slice_end;

                    let mut update_pointers = || {
                        curr_position += 1;
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
                AbstractMatchingExpression::Capture {
                    identifier,
                    identifier_type,
                } => match identifier_type {
                    CaptureType::Digit => {
                        let ch = input.as_bytes()[curr_position];
                        let ch_str = &input[curr_position..curr_position + 1];

                        if ch.is_ascii_digit() {
                            curr_position += 1;
                            state += 1;
                            captures_map.insert(identifier.as_ref(), ch_str);
                        } else {
                            curr_position += 1;
                            state = 0;
                        }
                    }
                    CaptureType::Int => {
                        let ch = input.as_bytes()[curr_position] as char;

                        let mut capture = |start: usize, curr_position: usize| {
                            captures_map.insert(identifier.as_ref(), &input[start..curr_position]);
                        };

                        if ch.is_ascii_digit() {
                            if let None = capture_slice_start {
                                capture_slice_start = Some(curr_position);
                                if state == 0 {
                                    legit_start = curr_position;
                                }
                            }

                            capture_candidate_found = Some(true);
                            curr_position += 1;

                            if curr_position == input.len() {
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
            return Some(Match {
                text: input,
                start: legit_start,
                end: curr_position,
            });
        }

        None
    }

    pub fn find_iter<'m: 'a, 't>(&'m self, text: &'t str) -> Matches<'t, 'm> {
        Matches::new(self, text)
    }
}

#[derive(Debug)]
pub struct Matches<'t, 'm> {
    pub(crate) text: &'t str,
    pub(crate) mex: &'m MatchExpression<'m>,
    last_end: usize,
}

impl<'t, 'm> Matches<'t, 'm> {
    pub fn new(mex: &'m MatchExpression<'m>, text: &'t str) -> Self {
        Self {
            text,
            mex,
            last_end: 0,
        }
    }
}

impl<'t: 'm, 'm> Iterator for Matches<'t, 'm> {
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
mod tests {

    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    #[test]
    fn match_counts() {
        macro_rules! assert_match_on {
            ($pattern:literal, $input:literal) => {
                let exp = Parser::new(Lexer::new($pattern)).parse_match_exp().unwrap();
                assert!(Matches::new(&exp, $input).count() > 0);
            };
            ($pattern:literal, $input:literal, $boolean:literal) => {
                let exp = Parser::new(Lexer::new($pattern)).parse_match_exp().unwrap();
                assert_eq!(Matches::new(&exp, $input).count() > 0, $boolean);
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
        assert_eq!(exp.get_capture("n").unwrap(), "321");
        assert_eq!(exp.get_capture("i").unwrap(), "78");
    }

    #[test]
    fn digit_capture_group() {
        let exp = Parser::new(Lexer::new("digit(d:dig)"))
            .parse_match_exp()
            .unwrap();
        let text = "aewrdigit276yoypa";

        assert_eq!(exp.find_at(text, 0).unwrap().as_str(), "digit2");
        assert_eq!(exp.get_capture("d").unwrap(), "2");
    }

    #[test]
    fn three_capture_groups() {
        let exp = Parser::new(Lexer::new("ab(n:int)love(i:int)ly(d:dig)"))
            .parse_match_exp()
            .unwrap();
        let text = "ab321love78ly8";

        assert_eq!(exp.find_at(text, 0).unwrap().as_str(), text);
        assert_eq!(exp.get_capture("n").unwrap(), "321");
        assert_eq!(exp.get_capture("i").unwrap(), "78");
        assert_eq!(exp.get_capture("d").unwrap(), "8");
    }

    #[test]
    fn int_capture_group_at_the_begining() {
        let exp = Parser::new(Lexer::new("(n:int)love(i:int)ly(d:dig)"))
            .parse_match_exp()
            .unwrap();
        let text = "ab321love78ly8";

        assert_eq!(exp.find_at(text, 0).unwrap().as_str(), &text[2..]);
        assert_eq!(exp.get_capture("n").unwrap(), "321");
        assert_eq!(exp.get_capture("i").unwrap(), "78");
        assert_eq!(exp.get_capture("d").unwrap(), "8");
    }

    #[test]
    fn special() {
        let mut parser = Parser::from_input("hello(as:dig)->oh(as)hi");
        let exp = parser.parse_match_exp().unwrap();
        assert_eq!(exp.find_at("ashello090", 0).unwrap().as_str(), "hello0");
    }

    #[test]
    fn muliple_matches() {
        let mut parser = Parser::from_input("xy(n:int)");
        let pattern = parser.parse_match_exp().unwrap();
        let text = "wxy10xy33asdfxy81";
        let mut matches = Matches::new(&pattern, text);

        assert_eq!(matches.next().unwrap().as_str(), "xy10");
        assert_eq!(matches.next().unwrap().as_str(), "xy33");
        assert_eq!(matches.next().unwrap().as_str(), "xy81");
    }
}
