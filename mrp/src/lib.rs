#![feature(pattern)]

mod error;
mod lexer;
pub mod parser;
mod pattern;

use std::str::{
    pattern::{Pattern, SearchStep, Searcher},
    FromStr,
};

use parser::MatchExpression;

pub struct MrpSearcher<'t> {
    haystack: &'t str,
    next_match: Option<(usize, usize)>,
}

impl<'p, 't> Pattern<'t> for &'p MatchExpression {
    type Searcher = MrpSearcher<'t>;

    fn into_searcher(self, haystack: &'t str) -> Self::Searcher {
        let mut finder = pattern::MatchFinder::new(&self, haystack);
        MrpSearcher {
            haystack,
            next_match: finder.next(),
        }
    }
}

unsafe impl<'p, 't> Searcher<'t> for MrpSearcher<'t> {
    #[inline]
    fn haystack(&self) -> &'t str {
        self.haystack
    }

    #[inline]
    fn next(&mut self) -> SearchStep {
        if let Some((s, e)) = self.next_match {
            self.next_match = None;
            return SearchStep::Match(s, e);
        }

        SearchStep::Done
    }
}

#[test]
fn exact_match() {
    let text = "abc235";
    let pattern = MatchExpression::from_str("abc(n:int)").unwrap();

    assert_eq!(text.find(&pattern).unwrap(), 0);
    assert_eq!(text.matches(&pattern).next(), Some("abc235"));
    assert_eq!(text.matches(&pattern).count(), 1);
}

#[test]
fn one_substr_with_extra_end_match() {
    let text = "abc235as";
    let pattern = MatchExpression::from_str("abc(n:int)").unwrap();

    assert_eq!(text.find(&pattern).unwrap(), 0);
    assert_eq!(text.contains(&pattern), true);
    assert_eq!(text.matches(&pattern).next(), Some("abc235"));
    assert_eq!(text.matches(&pattern).count(), 1);
}

#[test]
fn one_substr_with_extra_at_beginning_match() {
    let text = "aaabc235";
    let pattern = MatchExpression::from_str("abc(n:int)").unwrap();

    assert_eq!(text.find(&pattern).unwrap(), 2);
    assert_eq!(text.contains(&pattern), true);
    assert_eq!(text.matches(&pattern).next(), Some("abc235"));
    assert_eq!(text.matches(&pattern).count(), 1);
}
