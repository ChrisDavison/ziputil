use anyhow::Result;

mod command;
mod filter;
mod utility;

use command::Command;
use filter::Filter;

fn main() {
    let (command, zipfile, filter) = match parse_args() {
        Ok((c, z, f)) => (c, z, f),
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    };

    let matches = if let Ok(Some(m)) = filter.filter_zip_by_name(&zipfile) {
        m
    } else {
        println!("No matching files in zipfile.");
        std::process::exit(1);
    };

    if let Err(e) = command.execute(&matches, zipfile) {
        println!("{}", e);
        std::process::exit(3);
    }
}

const USAGE: &str = "usage: ziputil <command> <zipfile> [args] [query]...

Utility for listing, cat-ing, or extracting specific files from a zip archive.

Arguments:
    zipfile         a zip archive
    -o --ordered    query terms must be matched IN ORDER
    -a --any        match ANY, rather than ALL, queries

Commands:
    list       display files in zip archive matching QUERY...
    view       cat files in zip archive matching QUERY...
    choose     extract files in zip archive matching QUERY...";

fn parse_args() -> Result<(Command, String, Filter)> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        println!("{}", USAGE);
        std::process::exit(0);
    }

    let ordered = pargs.contains(["-o", "--ordered"]);
    let any = pargs.contains(["-a", "--any"]);

    let command = match pargs.subcommand()?.as_deref() {
        Some("choose") => Command::Choose,
        Some("view") => Command::View,
        Some("list") => Command::List,
        Some(unrecognised) => {
            println!("Unrecognised command: {:#?}\n", unrecognised);
            println!("{}", USAGE);
            std::process::exit(1);
        }
        _ => {
            println!("{}", USAGE);
            std::process::exit(1);
        }
    };

    let zipfile: String = pargs.free_from_str()?;

    let query = pargs
        .finish()
        .iter()
        .map(|x| x.to_string_lossy().to_string())
        .collect();

    let filter = Filter::new(any, ordered, query);

    Ok((command, zipfile, filter))
}
