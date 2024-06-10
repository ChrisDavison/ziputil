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
}

#[derive(Debug, StructOpt)]
enum OptCommand {
    /// Cat files matching query to stdout (dumb)
    #[structopt(alias = "v")]
    View(CommonArgs),
    /// List files matching query
    #[structopt(alias = "l")]
    List(CommonArgs),
    /// Choose files to extract
    #[structopt(alias = "c")]
    Choose(CommonArgs),
    /// Extract all files
    #[structopt(aliases = &["e", "x"])]
    Extract(CommonArgs)
}

#[derive(Debug, StructOpt)]
struct CommonArgs {
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

fn main() {
    let opts = Opts::from_args();

    let (command, args) = match opts.command {
        OptCommand::View(args) => (Command::View, args),
        OptCommand::List(args) => (Command::List, args),
        OptCommand::Choose(args) => (Command::Choose, args),
        OptCommand::Extract(args) => (Command::Extract, args),
    };
    let ordered = args.ordered;
    let any = args.any;
    let query = args.query;
    let zipfile = args.zipfile;

    let filter = Filter::new(any, ordered, query);

    let matches = if let Ok(Some(m)) = filter.filter_zip_by_name(&zipfile) {
        m
    } else {
        println!("No matching files in zipfile.");
        std::process::exit(1);
    };

    if let Err(e) = command.execute(&matches, &zipfile) {
        println!("{}", e);
        std::process::exit(3);
    }
}
