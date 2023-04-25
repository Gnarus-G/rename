use std::{path::PathBuf, process::ExitCode};

use clap::{Args, Parser, Subcommand};
use mrp::{MatchAndReplaceStrategy, MatchAndReplacer};

#[derive(Parser, Debug)]
#[clap(author, version, about, setting = clap::AppSettings::DeriveDisplayOrder)]
/// A utility for renaming paths (files and directories) in bulk.
struct RenameArgs {
    #[clap(subcommand)]
    command: Command,

    /// pattern for the paths to rename.
    #[clap(global = true, long, conflicts_with = "paths")]
    glob: Option<String>,

    /// One or more paths to rename.
    #[clap(global = true)]
    paths: Vec<std::path::PathBuf>,

    /// Don't actually rename the files, instead just print each rename that would happen.
    #[clap(long, global = true)]
    dry_run: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Use a simple match-and-replace-protocol syntax. (e.g. "hello(n:int)->hi(n)")
    SIMPLE(SimpleArgs),
    /// Use and apply a regex replace on each filename
    REGEX(RegexArgs),
}

fn main() -> ExitCode {
    let base_args = RenameArgs::parse();

    let paths = if let Some(aw) = &base_args.glob {
        glob::glob(aw)
            .expect("invalid glob pattern")
            .flatten()
            .collect()
    } else {
        base_args.paths
    };

    match base_args.command {
        Command::REGEX(ref args) => handle_regex_replacement(&args, &paths, base_args.dry_run),
        Command::SIMPLE(ref args) => match mrp::parser::Parser::from(&args.expression).parse() {
            Ok(ref e) => {
                let mut replacer = MatchAndReplacer::new(e);
                replacer.set_strip(args.strip);
                handle_mrp_replacement(&paths, replacer, base_args.dry_run);
            }
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        },
    };

    ExitCode::SUCCESS
}

#[derive(Debug, Args)]
struct SimpleArgs {
    /// A Match & Replace expression in the custom MRP syntax.
    expression: String,
    /// Strip off anything not explicitly matched for while replacting.
    #[clap(short, long)]
    strip: bool,
}

fn handle_mrp_replacement<'e>(paths: &'e [PathBuf], replacer: MatchAndReplacer<'e>, dry_run: bool) {
    paths
        .iter()
        .filter_map(|p| {
            let str = p.to_str();

            if str.is_none() {
                eprintln!("Path is invalid unicode: {:?}", p);
            }

            return str;
        })
        .map(|p| (p, replacer.apply(p)))
        .filter_map(|(from, to)| to.map(|t| (from, t)))
        .for_each(|(from, to)| {
            if dry_run {
                println!("Rename {:?} to {:?}", from, to);
            } else {
                if let Err(err) = std::fs::rename(from, to.to_string()) {
                    eprintln!("{:?}: {}", from, err);
                }
            }
        });
}

#[derive(Debug, Args, Clone)]
struct RegexArgs {
    /// The regex pattern with which to search.
    pattern: regex::Regex,
    /// The replacement format based on the regex capture groups.
    replacement: String,
}

fn handle_regex_replacement(args: &RegexArgs, paths: &[PathBuf], dry_run: bool) {
    let transform = |name| {
        return (
            name,
            args.pattern.replace(name, &args.replacement).to_string(),
        );
    };

    paths
        .iter()
        .for_each(|path| match path.to_str().map(transform) {
            None => eprintln!("Path is invalid unicode: {:?}", path),
            Some((from, to)) => {
                if dry_run {
                    println!("Rename {:?} to {:?}", path, to);
                } else {
                    if let Err(err) = std::fs::rename(from, to) {
                        eprintln!("{}: {}", from, err);
                    }
                }
            }
        })
}
