mod captures;
mod error;
pub mod lexer;
mod matcher;
pub mod parser;

use std::borrow::Cow;

use parser::{AbstractReplaceExpression, MatchAndReplaceExpression, MatchExpression};

/// Representing a stragety by which to match and replace on a `string` value
pub trait MatchAndReplaceStrategy<'input> {
    /// Match and replace
    fn apply(&self, value: &'input str) -> Option<std::borrow::Cow<'input, str>>;
}

pub struct MatchAndReplacer<'source> {
    mex: MatchExpression<'source>,
    exprs: Vec<AbstractReplaceExpression<'source>>,
    /// When true, this strategy will replace the matching range found, and strip everything else
    /// off.
    strip: bool,
}

impl<'source> MatchAndReplacer<'source> {
    pub fn new(mrex: MatchAndReplaceExpression<'source>) -> Self {
        Self {
            mex: mrex.mex,
            exprs: mrex.rex.expressions,
            strip: false,
        }
    }

    pub fn set_strip(&mut self, s: bool) {
        self.strip = s;
    }
}

impl<'input> MatchAndReplaceStrategy<'input> for MatchAndReplacer<'input> {
    fn apply(&self, value: &'input str) -> Option<std::borrow::Cow<'input, str>> {
        match self.mex.find_at_capturing(value, 0) {
            (None, _) => None,
            (Some(m), captures) => {
                let mut new = Cow::from(value);
                let replacement_str: String = self
                    .exprs
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
    use std::str::FromStr;

    use super::*;

    impl<'source> MatchAndReplacer<'source> {
        fn apply_all(&mut self, values: Vec<&'source str>) -> Vec<String> {
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
        let expression = MatchAndReplaceExpression::from_str(input).unwrap();
        let strat = MatchAndReplacer::new(expression);

        assert_eq!(strat.apply("lit12").unwrap(), "lul12");
    }

    #[test]
    fn test_mrp_application() {
        let input = "(num:int)asdf->lul(num)";
        let expression = MatchAndReplaceExpression::from_str(input).unwrap();
        let mut strat = MatchAndReplacer::new(expression);

        let treated = strat.apply_all(vec!["124asdf", "3asdfwery", "lk234asdfas"]);

        assert_eq!(treated, vec!["lul124", "lul3wery", "lklul234as"]);

        let expression = MatchAndReplaceExpression::from_str("hello(as:dig)->oh(as)hi").unwrap();

        let mut strat = MatchAndReplacer::new(expression);

        let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

        assert_eq!(treated, vec!["oh5hi", "asoh0hi90", "oh3hi45hello"]);
    }

    #[test]
    fn test_mrp_application_stripping() {
        let expression = MatchAndReplaceExpression::from_str("hello(as:dig)->oh(as)hi").unwrap();

        let mut strat = MatchAndReplacer::new(expression);

        strat.set_strip(true);

        let treated = strat.apply_all(vec!["hello5", "ashello090", "hello345hello"]);

        assert_eq!(treated, vec!["oh5hi", "oh0hi", "oh3hi"]);
    }

    #[test]
    fn test_mrp_application_with_multi_digits_and_stripping() {
        let expression = MatchAndReplaceExpression::from_str("(n:int)->step(n)").unwrap();
        let mut strat = MatchAndReplacer::new(expression);

        strat.set_strip(true);

        let treated = strat.apply_all(vec!["f1", "f11", "f99"]);

        assert_eq!(treated, vec!["step1", "step11", "step99"]);
    }

    #[test]
    fn handles_byte_indexing_inside_a_unicode_character() {
        let cases = [
            // control: doesn't cause panic, since capture group is far
            // away from the 〰
            ("a2—a—〰", "2b—a—〰"),
            // rest would other wise cause panic if we weren't carefull about indexing a &str
            ("as—a—a9", "as—a—9b"),
            ("a7as—a—〰", "7bas—a—〰"),
            ("a〰a4〰-34", "a〰4b〰-34"),
        ];

        for (input, output) in cases {
            let exp = MatchAndReplaceExpression::from_str("a(a:dig)->(a)b").unwrap();
            let strat = MatchAndReplacer::new(exp);

            assert_eq!(strat.apply(input).unwrap(), output);
        }
    }
}
