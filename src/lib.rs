#![feature(test)]
extern crate test;

use std::path::PathBuf;

use mrp::MatchAndReplaceStrategy;

pub struct BulkRenameOptions {
    pub no_rename: bool,
}

pub fn in_bulk<'p: 'r, 'r, R: MatchAndReplaceStrategy<'r>>(
    paths: &'p [PathBuf],
    rename: &mut R,
    options: &BulkRenameOptions,
) {
    let values = paths.iter().filter_map(|p| {
        let str = p.to_str();

        if str.is_none() {
            eprintln!("Path is invalid unicode: {:?}", p);
        }

        return str;
    });

    for from in values {
        if let Some(to) = rename.apply(from) {
            if options.no_rename {
                println!("{:?} -> {:?}", from, to);
            } else {
                if let Err(err) = std::fs::rename(from, to.to_string()) {
                    eprintln!("{:?}: {}", from, err);
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use mrp::{parser::Parser, MatchAndReplacer};

    use crate::in_bulk;

    fn create_files(count: usize) -> Vec<PathBuf> {
        fs::create_dir("./files").unwrap();

        let paths = (0..count)
            .map(|i| PathBuf::from(format!("./files/g-{i}-a-{i}-al-{i}")))
            .collect::<Vec<_>>();

        for path in paths.iter() {
            fs::File::create(path).unwrap();
        }

        return paths;
    }

    #[bench]
    fn bulk_rename(b: &mut test::Bencher) {
        let files = create_files(100000);

        b.iter(|| {
            let exp =
                Parser::from("g-(g:int)-a-(a:int)-al-(al:int)->artist-(a)-album-(al)-genre-(g)")
                    .parse()
                    .unwrap();

            let mut renamer = MatchAndReplacer::new(exp);
            in_bulk(
                &files,
                &mut renamer,
                &crate::BulkRenameOptions { no_rename: false },
            )
        });

        fs::remove_dir_all("./files").unwrap()
    }
}
