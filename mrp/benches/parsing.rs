use criterion::{criterion_group, criterion_main, Criterion};
use mrp::{
    lexer::{Lexer, Token},
    parser::Parser,
};

const EXPRESSION: &str = "aywer(n:dig)(num:int)asdf(lawerasdf:int)->lul(num)(n)asd(lasdkjf)(weoyr)";

fn lexing_benchmark(c: &mut Criterion) {
    let mut lexer = Lexer::new(EXPRESSION);
    c.bench_function("lexing", |b| {
        b.iter(|| {
            let mut t = lexer.next();
            while t != Token::End {
                t = lexer.next();
            }
        })
    });
}

fn parsing_benchmark(c: &mut Criterion) {
    let lexer = Lexer::new(EXPRESSION);
    let mut parser = Parser::new(lexer);
    c.bench_function("parsing", |b| {
        b.iter(|| {
            parser.parse_match_exp().unwrap();
            parser.parse_replacement_exp().unwrap()
        })
    });
}

criterion_group!(benches, lexing_benchmark, parsing_benchmark);
criterion_main!(benches);
