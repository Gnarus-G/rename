use std::path::PathBuf;

use mrp::MatchAndReplaceStrategy;

#[derive(Default, Clone)]
pub struct BulkRenameOptions {
    pub no_rename: bool,
    pub quiet: bool,
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
                if !options.quiet {
                    println!("{:?} -> {:?}", from, to);
                }
            } else {
                if let Err(err) = std::fs::rename(from, to.to_string()) {
                    eprintln!("{:?}: {}", from, err);
                }
            };
        }
    }
}
