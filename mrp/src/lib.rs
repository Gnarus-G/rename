mod captures;
mod error;
pub mod lexer;
mod matcher;
pub mod parser;

use std::{borrow::Cow, ops::Deref, sync::Arc};

use parser::{AbstractReplaceExpression, MatchAndReplaceExpression};

/// Representing a stragety by which to match and replace on a `string` value
pub trait MatchAndReplaceStrategy<'s> {
    /// Match and replace
    fn apply(&self, value: &'s str) -> Option<std::borrow::Cow<'s, str>>;
}

pub struct MatchAndReplacer<'m> {
    mrex: MatchAndReplaceExpression<'m>,
    /// When true, this strategy will replace the matching range found, and strip everything else
    /// off.
    strip: bool,
}

impl<'m> MatchAndReplacer<'m> {
    pub fn new(mrex: MatchAndReplaceExpression<'m>) -> Self {
        Self { mrex, strip: false }
    }

    pub fn set_strip(&mut self, s: bool) {
        self.strip = s;
    }
}

impl<'i> MatchAndReplaceStrategy<'i> for Arc<MatchAndReplacer<'i>> {
    fn apply(&self, value: &'i str) -> Option<std::borrow::Cow<'i, str>> {
        self.deref().apply(value)
    }
}

impl<'i> MatchAndReplaceStrategy<'i> for MatchAndReplacer<'i> {
    fn apply(&self, value: &'i str) -> Option<std::borrow::Cow<'i, str>> {
        match self.mrex.mex.find_at_capturing(value, 0) {
            (None, _) => None,
            (Some(m), captures) => {
                let mut new = Cow::from(value);
                let replacement_str: String = self
                    .mrex
                    .rex
                    .expressions
                    .iter()
                    .map(|e| match e {
                        AbstractReplaceExpression::Literal(l) => *l,
                        AbstractReplaceExpression::Identifier(i) => captures
                            .get(i)
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

#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    use super::*;

    impl<'m> MatchAndReplacer<'m> {
        fn apply_all(&mut self, values: Vec<&'m str>) -> Vec<String> {
            let mut replaced = vec![];
            for value in values {
                if let Some(v) = self.apply(value) {
                    replaced.push(v.to_string())
                }
            }

            return replaced;
        }
    }

    #[test]
    fn one_literal_and_int_capture() {
        let input = "lit(num:int)->lul(num)";
        let expression = Parser::from(input).parse().unwrap();
        let strat = MatchAndReplacer::new(expression);

        assert_eq!(strat.apply("lit12").unwrap(), "lul12");
    }

    #[test]
    fn test_mrp_application() {
        let input = "(num:int)asdf->lul(num)";
        let expression = Parser::from(input).parse().unwrap();
        let mut strat = MatchAndReplacer::new(expression);

        let treated = strat.apply_all(vec!["124asdf", "3asdfwery", "lk234asdfas"]);

        assert_eq!(treated, vec!["lul124", "lul3wery", "lklul234as"]);

        let expression = Parser::from("hello(as:dig)->oh(as)hi").parse().unwrap();

        let mut strat = MatchAndReplacer::new(expression);

        let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

        assert_eq!(treated, vec!["oh5hi", "asoh0hi90", "oh3hi45hello"]);
    }

    #[test]
    fn test_mrp_application_stripping() {
        let expression = Parser::from("hello(as:dig)->oh(as)hi").parse().unwrap();

        let mut strat = MatchAndReplacer::new(expression);

        strat.set_strip(true);

        let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

        assert_eq!(treated, vec!["oh5hi", "oh0hi", "oh3hi"]);
    }

    #[test]
    fn test_mrp_application_with_multi_digits_and_stripping() {
        let expression = Parser::from("(n:int)->step(n)").parse().unwrap();
        let mut strat = MatchAndReplacer::new(expression);

        strat.set_strip(true);

        let treated = strat.apply_all(vec!["f1", "f11", "f99"]);

        assert_eq!(treated, vec!["step1", "step11", "step99"]);
    }
}
