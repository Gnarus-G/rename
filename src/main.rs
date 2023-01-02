use clap::{Args, Parser, Subcommand};
use mrp::parser::MatchAndReplaceExpression;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
/// A utility for renaming paths (files and directories) in bulk.
struct RenameArgs {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Use a simple match-and-replace-protocol syntax. (e.g. "hello(n:int)->hi(n)")
    SIMPLE(SimpleArgs),
    /// Use and apply a regex replace on each filename
    REGEX(RegexArgs),
}

fn main() {
    let base_args = RenameArgs::parse();

    match &base_args.command {
        Command::REGEX(args) => handle_regex_replacement(args),
        Command::SIMPLE(args) => handle_mrp_replacement(args),
    }
}

#[derive(Debug, Args)]
struct SimpleArgs {
    /// A Match & Replace expression in the custom MRP syntax.
    expression: MatchAndReplaceExpression,

    /// One or more paths to rename.
    paths: Vec<std::path::PathBuf>,

    /// Don't actually rename the files, instead just print each rename that would happen.
    #[clap(long)]
    dry_run: bool,
}

fn handle_mrp_replacement(args: &SimpleArgs) {
    let path_strs = args
        .paths
        .iter()
        .filter_map(|p| {
            let str = p.to_str();
            if str.is_none() {
                eprintln!("Path is invalid unicode: {:?}", p);
            }
            return str;
        })
        .collect();

    let replacements = args.expression.apply(path_strs);

    args.paths.iter().zip(replacements).for_each(|(from, to)| {
        if args.dry_run {
            println!("Rename {:?} to {:?}", from, to);
        } else {
            if let Err(err) = std::fs::rename(from, to) {
                eprintln!("{:?}: {}", from, err);
            }
        }
    });
}

#[derive(Debug, Args)]
struct RegexArgs {
    /// The regex pattern with which to search.
    pattern: regex::Regex,
    /// The replacement format based on the regex capture groups.
    replacement: String,

    /// One or more paths to rename.
    paths: Vec<std::path::PathBuf>,

    /// Don't actually rename the files, instead just print each rename that would happen.
    #[clap(long)]
    dry_run: bool,
}

fn handle_regex_replacement(args: &RegexArgs) {
    let transform = |name| {
        return (
            name,
            args.pattern.replace(name, &args.replacement).to_string(),
        );
    };

    args.paths
        .iter()
        .for_each(|path| match path.to_str().map(transform) {
            None => eprintln!("Path is invalid unicode: {:?}", path),
            Some((from, to)) => {
                if args.dry_run {
                    println!("Rename {:?} to {:?}", path, to);
                } else {
                    if let Err(err) = std::fs::rename(from, to) {
                        eprintln!("{}: {}", from, err);
                    }
                }
            }
        })
}
