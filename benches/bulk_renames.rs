use rayon::prelude::*;
use std::{path::PathBuf, str::FromStr};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mrp::{MatchAndReplaceStrategy, MatchAndReplacer};

fn get_renamer() -> MatchAndReplacer<'static> {
    let expr = mrp::parser::MatchAndReplaceExpression::from_str(
        "g-(g:int)-a-(a:int)-al-(al:int)->artist-(a)-album-(al)-genre-(g)",
    )
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
                files.iter().filter_map(|p| p.to_str()).for_each(|name| {
                    renamer.apply(name);
                });
            });
        });
    }
}

fn comparing_rayon_and_single_threaded(c: &mut Criterion) {
    let renamer = get_renamer();
    let mut group = c.benchmark_group("rayon vs serial with a few files");
    group.sample_size(10);

    #[derive(Debug, Clone, Copy)]
    enum VS {
        Serial,
        Rayon,
    }

    for size in [
        (2, VS::Serial),
        (2, VS::Rayon),
        (20, VS::Serial),
        (20, VS::Rayon),
        (200, VS::Serial),
        (200, VS::Rayon),
        (20000, VS::Serial),
        (20000, VS::Rayon),
    ]
    .iter()
    {
        let files = create_file_paths(size.0);
        group.throughput(criterion::Throughput::Elements(size.0 as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{} with {:?}", size.0, size.1)),
            &(files, size.1),
            |b, (files, choice)| match choice {
                VS::Serial => {
                    b.iter(|| {
                        files.iter().filter_map(|p| p.to_str()).for_each(|name| {
                            renamer.apply(name);
                        });
                    });
                }
                VS::Rayon => {
                    b.iter(|| {
                        files
                            .par_iter()
                            .filter_map(|p| p.to_str())
                            .for_each(|name| {
                                renamer.apply(name);
                            });
                    });
                }
            },
        );
    }
}

criterion_group!(benches, renaming_files, comparing_rayon_and_single_threaded);
criterion_main!(benches);
