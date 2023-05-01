use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use mrp::{parser::MatchAndReplaceExpression, MatchAndReplaceStrategy, MatchAndReplacer};

#[derive(Parser, Debug)]
#[clap(author, version, about, setting = clap::AppSettings::DeriveDisplayOrder)]
/// A utility for renaming paths (files and directories) in bulk.
struct RenameArgs {
    #[clap(subcommand)]
    command: Command,

    /// Pattern for the paths to rename.
    #[clap(global = true, long, conflicts_with = "paths")]
    glob: Option<String>,

    /// Enable multi-threading to process more files at a time
    #[clap(global = true, short, long = "multi")]
    multithreading: bool,

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

    let options = &rename::BulkRenameOptions {
        no_rename: base_args.dry_run,
        quiet: false,
    };

    match base_args.command {
        Command::REGEX(args) => rename::in_bulk(&paths, &args, options, base_args.multithreading),
        Command::SIMPLE(args) => {
            let mut replacer = MatchAndReplacer::new(args.expression);
            replacer.set_strip(args.strip);
            rename::in_bulk(&paths, &mut replacer, options, base_args.multithreading);
        }
    };

    ExitCode::SUCCESS
}

#[derive(Debug, Args)]
struct SimpleArgs {
    /// A Match & Replace expression in the custom MRP syntax.
    expression: MatchAndReplaceExpression<'static>,
    /// Strip off anything not explicitly matched for while replacting.
    #[clap(short, long)]
    strip: bool,
}

#[derive(Debug, Args, Clone)]
struct RegexArgs {
    /// The regex pattern with which to search.
    pattern: regex::Regex,
    /// The replacement format based on the regex capture groups.
    replacement: String,
}

impl<'s> MatchAndReplaceStrategy<'s> for RegexArgs {
    fn apply(&self, value: &'s str) -> Option<std::borrow::Cow<'s, str>> {
        Some(self.pattern.replace(value, self.replacement.clone()))
    }
}
