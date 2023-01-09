mod error;
mod lexer;
mod matcher;
pub mod parser;

use std::{borrow::Cow, str::FromStr};

use error::ParseError;
use lexer::{Lexer, Token};
use parser::{
    AbstractMatchingExpression, AbstractReplaceExpression, MatchExpression, Parser,
    ReplaceExpression,
};

pub trait MRP {
    fn apply<'sf, 's: 'sf>(&'sf self, value: &'s str) -> Option<std::borrow::Cow<str>>;
}

#[derive(Debug, PartialEq)]
pub struct MatchAndReplaceExpression {
    mex: MatchExpression,
    rex: ReplaceExpression,
}

impl FromStr for MatchAndReplaceExpression {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut p = Parser::new(Lexer::new(s));
        Ok(MatchAndReplaceExpression {
            mex: p.parse_match_exp()?,
            rex: p.parse_replacement_exp()?,
        })
    }
}

pub struct InHouseStrategy<'m> {
    mrex: &'m MatchAndReplaceExpression,
}

impl<'m> InHouseStrategy<'m> {
    pub fn new(mrex: &'m MatchAndReplaceExpression) -> Self {
        Self { mrex }
    }
}

impl<'m> MRP for InHouseStrategy<'m> {
    fn apply<'sf, 's: 'sf>(&'sf self, value: &'s str) -> Option<std::borrow::Cow<str>> {
        match self.mrex.mex.find_at(value, 0) {
            None => None,
            Some(m) => {
                let mut new = Cow::from(value);
                let replacement_str: String = self
                    .mrex
                    .rex
                    .expressions
                    .iter()
                    .map(|e| match e {
                        AbstractReplaceExpression::Literal(l) => l.to_owned(),
                        AbstractReplaceExpression::Identifier(i) => self
                            .mrex
                            .mex
                            .get_capture(i)
                            .expect("should have been captured"),
                    })
                    .collect();

                new.to_mut().replace_range(m.start..m.end, &replacement_str);
                Some(new)
            }
        }
    }
}

#[derive(Debug)]
pub struct RegexTranspilationStrategy {
    regex: regex::Regex,
    replacement: String,
}

impl RegexTranspilationStrategy {
    pub fn new(mrex: &MatchAndReplaceExpression) -> Self {
        let (match_exp, replace_exp) = (&mrex.mex, &mrex.rex);
        let regex_pattern: String = match_exp
            .expressions
            .iter()
            .filter_map(|e| match e {
                AbstractMatchingExpression::Literal(l) => Some(l.clone()),
                AbstractMatchingExpression::Capture { identifier, typing } => {
                    if let Token::Ident(id) = identifier {
                        return match typing {
                            Token::DigitType => Some(format!("(?P<{id}>\\d)")),
                            Token::IntType => Some(format!("(?P<{id}>\\d+)")),
                            _ => None,
                        };
                    };

                    None
                }
            })
            .collect();

        let regex_replacement: String = replace_exp
            .expressions
            .iter()
            .filter_map(|e| match e {
                AbstractReplaceExpression::Literal(l) => Some(l.clone()),
                AbstractReplaceExpression::Identifier(id) => Some(format!("${{{id}}}")),
            })
            .collect();

        Self {
            regex: regex::Regex::new(&regex_pattern).expect("should be a valid regular expression"),
            replacement: regex_replacement,
        }
    }

    pub fn make_pattern_strip_non_matched_parts(&mut self) {
        let mut regex = self.regex.to_string();
        regex.insert_str(0, ".*?");
        regex.push_str(".*");

        self.regex = regex::Regex::new(&regex).unwrap();
    }
}

impl MRP for RegexTranspilationStrategy {
    fn apply<'sf, 's: 'sf>(&'sf self, value: &'s str) -> Option<std::borrow::Cow<str>> {
        let pattern = &self.regex;

        if !pattern.is_match(value) {
            return None;
        }
        return Some(pattern.replace(value, &self.replacement));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'m> InHouseStrategy<'m> {
        fn apply_all<'sf, 's: 'sf>(&'s self, values: Vec<&'s str>) -> Vec<std::borrow::Cow<str>> {
            return values.iter().filter_map(|s| self.apply(s)).collect();
        }
    }

    #[test]
    fn one_literal_and_int_capture() {
        let input = "lit(num:int)->lul(num)";
        let expression = MatchAndReplaceExpression::from_str(input).unwrap();
        let strat = InHouseStrategy::new(&expression);

        assert_eq!(strat.apply("lit12").unwrap(), "lul12");
    }

    #[test]
    fn test_mrp_application() {
        let input = "(num:int)asdf->lul(num)";
        let expression = MatchAndReplaceExpression::from_str(input).unwrap();
        let strat = InHouseStrategy::new(&expression);

        let treated = strat.apply_all(vec!["124asdf", "3asdfwery", "lk234asdfas"]);

        assert_eq!(treated, vec!["lul124", "lul3wery", "lklul234as"]);

        let expression = MatchAndReplaceExpression::from_str("hello(as:dig)->oh(as)hi").unwrap();

        let strat = InHouseStrategy::new(&expression);

        let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

        assert_eq!(treated, vec!["oh5hi", "asoh0hi90", "oh3hi45hello"]);
    }

    mod regex_imp {
        use super::*;

        impl RegexTranspilationStrategy {
            fn apply_all<'sf, 's: 'sf>(
                &'s self,
                values: Vec<&'s str>,
            ) -> Vec<std::borrow::Cow<str>> {
                return values.iter().filter_map(|s| self.apply(s)).collect();
            }
        }

        #[test]
        fn test_mrp_application() {
            let input = "(num:int)asdf->lul(num)";
            let expression = RegexTranspilationStrategy::new(
                &MatchAndReplaceExpression::from_str(input).unwrap(),
            );

            let treated = expression.apply_all(vec!["124asdf", "3asdfwery", "lk234asdfas"]);

            assert_eq!(treated, vec!["lul124", "lul3wery", "lklul234as"]);

            let expression =
                MatchAndReplaceExpression::from_str("hello(as:dig)->oh(as)hi").unwrap();

            let strat = RegexTranspilationStrategy::new(&expression);

            let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

            assert_eq!(treated, vec!["oh5hi", "asoh0hi90", "oh3hi45hello"]);
        }

        #[test]
        fn test_mrp_application_stripping() {
            let expression =
                MatchAndReplaceExpression::from_str("hello(as:dig)->oh(as)hi").unwrap();
            let mut strat = RegexTranspilationStrategy::new(&expression);

            strat.make_pattern_strip_non_matched_parts();

            let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

            assert_eq!(treated, vec!["oh5hi", "oh0hi", "oh3hi"]);
        }

        #[test]
        fn test_mrp_application_with_multi_digits_and_stripping() {
            let expression = MatchAndReplaceExpression::from_str("(n:int)->step(n)").unwrap();
            let mut strat = RegexTranspilationStrategy::new(&expression);

            strat.make_pattern_strip_non_matched_parts();

            let treated = strat.apply_all(vec!["f1", "f11", "f99"]);

            assert_eq!(treated, vec!["step1", "step11", "step99"]);
        }
    }
}
