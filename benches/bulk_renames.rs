use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mrp::{parser::Parser, MatchAndReplacer};
use rename::{in_bulk, BulkRenameOptions};

fn get_renamer() -> MatchAndReplacer<'static> {
    let expr = Parser::from("g-(g:int)-a-(a:int)-al-(al:int)->artist-(a)-album-(al)-genre-(g)")
        .parse()
        .unwrap();

    return MatchAndReplacer::new(expr);
}

fn create_file_paths(count: usize) -> Vec<PathBuf> {
    let paths = (0..count)
        .map(|i| PathBuf::from(format!("./files/g-{i}-a-{i}-al-{i}")))
        .collect::<Vec<_>>();

    return paths;
}

fn renaming_files(c: &mut Criterion) {
    let renamer = get_renamer();
    let mut group = c.benchmark_group("renames");
    group.sample_size(10);

    for size in [10, 100, 1000, 10000, 100000, 1000000].iter() {
        let files = create_file_paths(*size);
        group.throughput(criterion::Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), &files, |b, files| {
            b.iter(|| {
                in_bulk(
                    &files,
                    &renamer,
                    &BulkRenameOptions {
                        no_rename: true,
                        quiet: true,
                    },
                    false,
                )
            });
        });
    }
}

fn renaming_files_in_parallel(c: &mut Criterion) {
    let renamer = get_renamer();
    let mut group = c.benchmark_group("renames in parallel");
    group.sample_size(10);

    for size in [1000, 10000, 100000, 1000000].iter() {
        let files = create_file_paths(*size);
        group.throughput(criterion::Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), &files, |b, files| {
            b.iter(|| {
                in_bulk(
                    &files,
                    &renamer,
                    &BulkRenameOptions {
                        no_rename: true,
                        quiet: true,
                    },
                    true,
                )
            });
        });
    }
}

criterion_group!(benches, renaming_files, renaming_files_in_parallel);
criterion_main!(benches);
