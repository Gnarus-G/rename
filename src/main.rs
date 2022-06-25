use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
/// A utility for renaming files in bulk, apply a regex replace on each filename.
struct Args {
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

fn main() {
    let args = Args::parse();
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
