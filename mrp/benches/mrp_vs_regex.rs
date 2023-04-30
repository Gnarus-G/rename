use criterion::{criterion_group, criterion_main, Criterion};
use mrp::{parser::Parser, MatchAndReplaceStrategy, MatchAndReplacer};
use regex::Regex;

const EXP: &str = "(num:int)asdf->lul(num)";
const INPUT: &str = "lk234asdfas";

fn regex_benchmark(c: &mut Criterion) {
    let r = Regex::new("(\\d+)asdf").unwrap();
    c.bench_function("regex replace", |b| b.iter(|| r.replace(INPUT, "lul$1")));
}

fn mrp_benchmark(c: &mut Criterion) {
    let exp = Parser::from(EXP).parse().unwrap();
    let mut r = MatchAndReplacer::new(exp);
    c.bench_function("mrp strat", |b| {
        b.iter(|| {
            r.apply(INPUT);
        })
    });
}

criterion_group!(benches, regex_benchmark, mrp_benchmark);
criterion_main!(benches);
