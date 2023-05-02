use std::{path::PathBuf, thread};

use log::*;
use mrp::MatchAndReplaceStrategy;

#[derive(Default)]
pub struct BulkRenameOptions {
    pub no_rename: bool,
}

pub fn in_bulk<'p: 'r, 'r, R: MatchAndReplaceStrategy<'r> + std::marker::Sync>(
    paths: &'p [PathBuf],
    rename: &R,
    options: &BulkRenameOptions,
    multi: bool,
) {
    if paths.is_empty() {
        return;
    }

    if multi {
        let thread_count = num_cpus::get();

        if thread_count > paths.len() {
            warn!("there are more threads that files to rename, so single threaded it is");
        } else if thread_count * 500 > paths.len() {
            warn!("probably too few files to warrant multithreading, but here we go...");
            return in_bulk_multithreaded(paths, rename, thread_count, options);
        } else {
            return in_bulk_multithreaded(paths, rename, thread_count, options);
        }
    }
    return in_bulk_single_thread(paths, rename, options);
}

fn in_bulk_single_thread<'p: 'r, 'r, R: MatchAndReplaceStrategy<'r>>(
    paths: &'p [PathBuf],
    rename: &R,
    options: &BulkRenameOptions,
) {
    let values = paths.iter().filter_map(|p| {
        let str = p.to_str();

        if str.is_none() {
            error!("Path is invalid unicode: {:?}", p);
        }

        return str;
    });

    for from in values {
        if let Some(to) = rename.apply(from) {
            if options.no_rename {
                println!("{:?} -> {:?}", from, to);
            } else {
                if let Err(err) = std::fs::rename(from, to.to_string()) {
                    error!("{:?}: {}", from, err);
                }
            };
        }
    }
}

fn in_bulk_multithreaded<'p: 'r, 'r, R: MatchAndReplaceStrategy<'r> + std::marker::Sync>(
    paths: &'p [PathBuf],
    rename: &R,
    thread_count: usize,
    options: &BulkRenameOptions,
) {
    debug!("found {} threads available on this machine", thread_count);
    let max_chunk_size = paths.len() / (thread_count - 1);

    debug!(
        "chunking work, to handle {} files in each of {} threads",
        max_chunk_size, thread_count
    );

    let chunks = paths.chunks(max_chunk_size);

    thread::scope(|s| {
        let mut join_handles = vec![];

        for (id, path_chunk) in chunks.enumerate() {
            if let Ok(handle) = thread::Builder::new().spawn_scoped(s, || {
                in_bulk_single_thread(path_chunk, rename, &options);
            }) {
                debug!(
                    "spawned thread {} with {} file to rename",
                    id,
                    path_chunk.len()
                );
                join_handles.push(handle);
            } else {
                error!(
                    "failed to spawn thread {}, renaming the next {} files in the main thread",
                    id,
                    path_chunk.len()
                );
                in_bulk_single_thread(path_chunk, rename, &options);
            };
        }

        for handle in join_handles {
            handle
                .join()
                .expect("Couldn't join on the associated thread")
        }
    })
}
