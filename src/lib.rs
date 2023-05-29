use std::path::PathBuf;

use log::*;
use mrp::MatchAndReplaceStrategy;
use rayon::prelude::*;

pub struct BulkRenameOptions {
    pub no_rename: bool,
}

pub fn in_bulk<'p: 'r, 'r, R: MatchAndReplaceStrategy<'r> + std::marker::Sync>(
    paths: &'p [PathBuf],
    rename: &R,
    options: &BulkRenameOptions,
) {
    paths
        .par_iter()
        .filter_map(|p| {
            let path_string = p.to_str();

            if path_string.is_none() {
                error!("Path is invalid unicode: {:?}", p);
            }

            return match path_string {
                Some(s) => rename.apply(s).map(|renamed| (s, renamed)),
                None => None,
            };
        })
        .for_each(|(from, to)| {
            if options.no_rename {
                println!("{:?} -> {:?}", from, to);
            } else if let Err(err) = std::fs::rename(from, to.to_string()) {
                error!("{:?}: {}", from, err);
            };
        })
}
