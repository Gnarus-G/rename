mod error;
pub mod lexer;
mod matcher;
pub mod parser;

use std::borrow::Cow;

use parser::{AbstractReplaceExpression, MatchAndReplaceExpression};

/// Representing a stragety by which to match and replace on a `string` value
pub trait MatchAndReplaceStrategy<'m> {
    /// Match and replace
    fn apply<'sf, 's: 'sf + 'm>(&'sf self, value: &'s str) -> Option<std::borrow::Cow<str>>;
}

pub struct MatchAndReplacer<'m> {
    mrex: &'m MatchAndReplaceExpression<'m>,
    /// When true, this strategy will replace the matching range found, and strip everything else
    /// off.
    strip: bool,
}

impl<'m> MatchAndReplacer<'m> {
    pub fn new(mrex: &'m MatchAndReplaceExpression<'m>) -> Self {
        Self { mrex, strip: false }
    }

    pub fn set_strip(&mut self, s: bool) {
        self.strip = s;
    }
}

impl<'m> MatchAndReplaceStrategy<'m> for MatchAndReplacer<'m> {
    fn apply<'sf, 's: 'sf + 'm>(&'sf self, value: &'s str) -> Option<std::borrow::Cow<str>> {
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
                        AbstractReplaceExpression::Literal(l) => *l,
                        AbstractReplaceExpression::Identifier(i) => self
                            .mrex
                            .mex
                            .get_capture(i)
                            .expect(&format!("'{i}' should have been captured")),
                        AbstractReplaceExpression::CaptureIndex(idx) => self
                            .mrex
                            .mex
                            .get_capture_index(*idx)
                            .expect(&format!("index '{idx}' should have been captured")),
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

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    use super::*;

    impl<'m> MatchAndReplacer<'m> {
        fn apply_all<'sf, 's: 'sf + 'm>(
            &'s self,
            values: Vec<&'s str>,
        ) -> Vec<std::borrow::Cow<str>> {
            return values.iter().filter_map(|s| self.apply(s)).collect();
        }
    }

    #[test]
    fn one_literal_and_int_capture() {
        let input = "lit(num:int)->lul(num)";
        let expression = Parser::from(input).parse().unwrap();
        let strat = MatchAndReplacer::new(&expression);

        assert_eq!(strat.apply("lit12").unwrap(), "lul12");
    }

    #[test]
    fn test_mrp_application() {
        let input = "(num:int)asdf->lul(num)";
        let expression = Parser::from(input).parse().unwrap();
        let strat = MatchAndReplacer::new(&expression);

        let treated = strat.apply_all(vec!["124asdf", "3asdfwery", "lk234asdfas"]);

        assert_eq!(treated, vec!["lul124", "lul3wery", "lklul234as"]);

        let expression = Parser::from("hello(as:dig)->oh(as)hi").parse().unwrap();

        let strat = MatchAndReplacer::new(&expression);

        let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

        assert_eq!(treated, vec!["oh5hi", "asoh0hi90", "oh3hi45hello"]);
    }

    #[test]
    fn test_mrp_application_stripping() {
        let expression = Parser::from("hello(as:dig)->oh(as)hi").parse().unwrap();
        let mut strat = MatchAndReplacer::new(&expression);

        strat.set_strip(true);

        let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

        assert_eq!(treated, vec!["oh5hi", "oh0hi", "oh3hi"]);
    }

    #[test]
    fn test_mrp_application_with_multi_digits_and_stripping() {
        let expression = Parser::from("(n:int)->step(n)").parse().unwrap();
        let mut strat = MatchAndReplacer::new(&expression);

        strat.set_strip(true);

        let treated = strat.apply_all(vec!["f1", "f11", "f99"]);

        assert_eq!(treated, vec!["step1", "step11", "step99"]);
    }
}
