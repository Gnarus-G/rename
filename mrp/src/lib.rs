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
use pattern::Matches;

pub struct MexSearcher<'t, 'm> {
    haystack: &'t str,
    it: Matches<'t, 'm>,
}

impl<'m: 't, 't> Pattern<'t> for &'m MatchExpression {
    type Searcher = MexSearcher<'t, 'm>;

    fn into_searcher(self, haystack: &'t str) -> Self::Searcher {
        MexSearcher {
            haystack,
            it: self.find_iter(haystack),
        }
    }
}

unsafe impl<'t, 'm> Searcher<'t> for MexSearcher<'t, 'm> {
    #[inline]
    fn haystack(&self) -> &'t str {
        self.haystack
    }

    #[inline]
    fn next(&mut self) -> SearchStep {
        match self.it.next() {
            Some((s, e)) => SearchStep::Match(s, e),
            None => SearchStep::Done,
        }
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

#[test]
fn multiple_matches() {
    let text = "aaabc235fnabc8iw6788abc9923";
    let pattern = MatchExpression::from_str("abc(n:int)").unwrap();
    let mut matches = text.matches(&pattern);

    assert_eq!(text.find(&pattern).unwrap(), 2);
    assert_eq!(text.contains(&pattern), true);
    assert_eq!(text.matches(&pattern).count(), 3);
    assert_eq!(matches.next(), Some("abc235"));
    assert_eq!(matches.next(), Some("abc8"));
    assert_eq!(matches.next(), Some("abc9923"));
}
