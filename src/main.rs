mod command;
mod filter;
mod utility;

use command::Command;
use filter::Filter;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "repoutil", about = "Operations on multiple git repos")]
struct Opts {
    #[structopt(subcommand)]
    command: OptCommand,
    /// query terms must be matched IN ORDER
    #[structopt(long, short)]
    ordered: bool,
    /// match ANY, rather than ALL, queries
    #[structopt(long, short)]
    any: bool,
    /// Zipfile to search
    zipfile: std::path::PathBuf,
    /// Query to match
    query: Vec<String>,
}

#[derive(Debug, StructOpt)]
enum OptCommand {
    /// Cat files matching query to stdout (dumb)
    #[structopt(alias = "v")]
    View,
    /// List files matching query
    #[structopt(alias = "l")]
    List,
    /// Choose files to extract
    #[structopt(alias = "c")]
    Choose,
    /// Extract all files
    #[structopt(aliases = &["e", "x"])]
    Extract,
}

fn main() {
    let opts = Opts::from_args();
    let ordered = opts.ordered;
    let any = opts.any;

    let command = match opts.command {
        OptCommand::View => Command::View,
        OptCommand::List => Command::List,
        OptCommand::Choose => Command::Choose,
        OptCommand::Extract => Command::Extract,
    };

    let filter = Filter::new(any, ordered, opts.query);

    let matches = if let Ok(Some(m)) = filter.filter_zip_by_name(&opts.zipfile) {
        m
    } else {
        println!("No matching files in zipfile.");
        std::process::exit(1);
    };

    if let Err(e) = command.execute(&matches, &opts.zipfile) {
        println!("{}", e);
        std::process::exit(3);
    }
}
