mod error;
pub mod lexer;
mod matcher;
pub mod parser;

use std::borrow::Cow;

use parser::{AbstractMatchingExpression, AbstractReplaceExpression, MatchAndReplaceExpression};

/// Representing a stragety by which to match and replace on a `string` value
pub trait MatchAndReplaceStrategy {
    /// Match and replace
    fn apply<'sf, 's: 'sf>(&'sf self, value: &'s str) -> Option<std::borrow::Cow<str>>;
}

pub struct DefaultMatchAndReplaceStrategy<'m> {
    mrex: &'m MatchAndReplaceExpression<'m>,
    /// When true, this strategy will replace the matching range found, and strip everything else
    /// off.
    strip: bool,
}

impl<'m> DefaultMatchAndReplaceStrategy<'m> {
    pub fn new(mrex: &'m MatchAndReplaceExpression<'m>) -> Self {
        Self { mrex, strip: false }
    }

    pub fn set_strip(&mut self, s: bool) {
        self.strip = s;
    }
}

impl<'m> MatchAndReplaceStrategy for DefaultMatchAndReplaceStrategy<'m> {
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
                        AbstractReplaceExpression::Literal(l) => l.to_string(),
                        AbstractReplaceExpression::Identifier(i) => self
                            .mrex
                            .mex
                            .get_capture(i)
                            .expect(&format!("'{i}' should have been captured")),
                    })
                    .collect();

                if self.strip {
                    new = Cow::from(replacement_str);
                } else {
                    new.to_mut().replace_range(m.start..m.end, &replacement_str);
                }

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
                AbstractMatchingExpression::Literal(l) => Some(l.to_string()),
                AbstractMatchingExpression::Capture {
                    identifier,
                    identifier_type,
                } => {
                    return match identifier_type {
                        parser::CaptureType::Digit => Some(format!("(?P<{}>\\d)", identifier)),
                        parser::CaptureType::Int => Some(format!("(?P<{}>\\d+)", identifier)),
                    };
                }
            })
            .collect();

        let regex_replacement: String = replace_exp
            .expressions
            .iter()
            .filter_map(|e| match e {
                AbstractReplaceExpression::Literal(l) => Some(l.to_string()),
                AbstractReplaceExpression::Identifier(id) => Some(format!("${{{}}}", *id)),
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

impl MatchAndReplaceStrategy for RegexTranspilationStrategy {
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
    use crate::parser::Parser;

    use super::*;

    impl<'m> DefaultMatchAndReplaceStrategy<'m> {
        fn apply_all<'sf, 's: 'sf>(&'s self, values: Vec<&'s str>) -> Vec<std::borrow::Cow<str>> {
            return values.iter().filter_map(|s| self.apply(s)).collect();
        }
    }

    #[test]
    fn one_literal_and_int_capture() {
        let input = "lit(num:int)->lul(num)";
        let expression = Parser::from_input(input).parse().unwrap();
        let strat = DefaultMatchAndReplaceStrategy::new(&expression);

        assert_eq!(strat.apply("lit12").unwrap(), "lul12");
    }

    #[test]
    fn test_mrp_application() {
        let input = "(num:int)asdf->lul(num)";
        let expression = Parser::from_input(input).parse().unwrap();
        let strat = DefaultMatchAndReplaceStrategy::new(&expression);

        let treated = strat.apply_all(vec!["124asdf", "3asdfwery", "lk234asdfas"]);

        assert_eq!(treated, vec!["lul124", "lul3wery", "lklul234as"]);

        let expression = Parser::from_input("hello(as:dig)->oh(as)hi")
            .parse()
            .unwrap();

        let strat = DefaultMatchAndReplaceStrategy::new(&expression);

        let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

        assert_eq!(treated, vec!["oh5hi", "asoh0hi90", "oh3hi45hello"]);
    }

    #[test]
    fn test_mrp_application_stripping() {
        let expression = Parser::from_input("hello(as:dig)->oh(as)hi")
            .parse()
            .unwrap();
        let mut strat = DefaultMatchAndReplaceStrategy::new(&expression);

        strat.set_strip(true);

        let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

        assert_eq!(treated, vec!["oh5hi", "oh0hi", "oh3hi"]);
    }

    #[test]
    fn test_mrp_application_with_multi_digits_and_stripping() {
        let expression = Parser::from_input("(n:int)->step(n)").parse().unwrap();
        let mut strat = DefaultMatchAndReplaceStrategy::new(&expression);

        strat.set_strip(true);

        let treated = strat.apply_all(vec!["f1", "f11", "f99"]);

        assert_eq!(treated, vec!["step1", "step11", "step99"]);
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
            let expression =
                RegexTranspilationStrategy::new(&Parser::from_input(input).parse().unwrap());

            let treated = expression.apply_all(vec!["124asdf", "3asdfwery", "lk234asdfas"]);

            assert_eq!(treated, vec!["lul124", "lul3wery", "lklul234as"]);

            let expression = Parser::from_input("hello(as:dig)->oh(as)hi")
                .parse()
                .unwrap();

            let strat = RegexTranspilationStrategy::new(&expression);

            let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

            assert_eq!(treated, vec!["oh5hi", "asoh0hi90", "oh3hi45hello"]);
        }

        #[test]
        fn test_mrp_application_stripping() {
            let expression = Parser::from_input("hello(as:dig)->oh(as)hi")
                .parse()
                .unwrap();
            let mut strat = RegexTranspilationStrategy::new(&expression);

            strat.make_pattern_strip_non_matched_parts();

            let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

            assert_eq!(treated, vec!["oh5hi", "oh0hi", "oh3hi"]);
        }

        #[test]
        fn test_mrp_application_with_multi_digits_and_stripping() {
            let expression = Parser::from_input("(n:int)->step(n)").parse().unwrap();
            let mut strat = RegexTranspilationStrategy::new(&expression);

            strat.make_pattern_strip_non_matched_parts();

            let treated = strat.apply_all(vec!["f1", "f11", "f99"]);

            assert_eq!(treated, vec!["step1", "step11", "step99"]);
        }
    }
}
