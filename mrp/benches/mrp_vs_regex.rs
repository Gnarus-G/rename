use criterion::{criterion_group, criterion_main, Criterion};
use mrp::{
    parser::Parser, DefaultMatchAndReplaceStrategy, MatchAndReplaceStrategy,
    RegexTranspilationStrategy,
};
use regex::Regex;

const EXP: &str = "(num:int)asdf->lul(num)";
const INPUT: &str = "lk234asdfas";

fn regex_benchmark(c: &mut Criterion) {
    let r = Regex::new("(\\d+)asdf").unwrap();
    c.bench_function("regex replace", |b| b.iter(|| r.replace(INPUT, "lul$1")));
}

fn regex_transpl_benchmark(c: &mut Criterion) {
    let exp = Parser::from(EXP).parse().unwrap();
    let r = RegexTranspilationStrategy::new(&exp);
    c.bench_function("regex transpile strat", |b| {
        b.iter(|| {
            r.apply(INPUT);
        })
    });
}

fn mrp_benchmark(c: &mut Criterion) {
    let exp = Parser::from(EXP).parse().unwrap();
    let r = DefaultMatchAndReplaceStrategy::new(&exp);
    c.bench_function("mrp strat", |b| {
        b.iter(|| {
            r.apply(INPUT);
        })
    });
}

criterion_group!(
    benches,
    regex_benchmark,
    regex_transpl_benchmark,
    mrp_benchmark
);
criterion_main!(benches);
