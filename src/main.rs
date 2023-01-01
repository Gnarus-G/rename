use clap::{Args, Parser, Subcommand};
use mrp::parser::MatchAndReplaceExpression;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
/// A utility for renaming files in bulk.
struct RenameArgs {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Use a simple match-and-replace-protocol syntax.
    SIMPLE {
        /// A Match & Replace expression in the custom MRP syntax.
        expression: MatchAndReplaceExpression,
        /// One or more files to rename.
        paths: Vec<std::path::PathBuf>,
    },
    /// Use and apply a regex replace on each filename
    REGEX(RegexArgs),
}

fn main() {
    let args = RenameArgs::parse();

    match args.command {
        Command::REGEX(args) => handle_regex_replacement(args),
        Command::SIMPLE { expression, paths } => {
            dbg!(expression, paths);
        }
    }
}

#[derive(Debug, Args)]
struct RegexArgs {
    /// The regex pattern with which to search.
    pattern: regex::Regex,
    /// The replacement format based on the regex capture groups.
    replacement: String,
    /// One or more files to rename.
    files: Vec<std::path::PathBuf>,

    #[clap(long)]
    /// Don't actually rename the files, instead just print each rename that would happen.
    dry_run: bool,
}

fn handle_regex_replacement(args: RegexArgs) {
    let transform = |name| {
        return (
            name,
            args.pattern.replace(name, &args.replacement).to_string(),
        );
    };

    args.files
        .iter()
        .for_each(|file| match file.to_str().map(transform) {
            None => eprintln!("Path is invalid unicode: {:?}", file),
            Some((from, to)) => {
                if args.dry_run {
                    println!("Rename {:?} to {:?}", file, to);
                } else {
                    if let Err(err) = std::fs::rename(from, to) {
                        eprintln!("{}: {}", from, err);
                    }
                }
            }
        })
}
