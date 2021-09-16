use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

mod filter;
mod utility;

use filter::Filter;

fn main() {
    if let Err(e) = try_main() {
        println!("{}", e);
    }
}

fn try_main() -> Result<()> {
    let (command, zipfile, filter) = parse_args()?;
    let matches = match filter.filter_zip_by_name(&zipfile)? {
        Some(m) => m,
        None => {
            println!("No matching files in zipfile.");
            std::process::exit(0);
        }
    };

    println!("Matches");
    for (i, m) in matches.iter().enumerate() {
        println!("{}. {}", i, m);
    }

    match command {
        // List is a noop, as matches get printed during filtering
        Command::List => {}
        Command::Choose => {
            let to_take = utility::choose_from_vector(&matches)?;
            let zip_name = PathBuf::from(&zipfile)
                .file_name()
                .context("Couldn't get zip filename")?
                .to_string_lossy()
                .to_string();
            let dirname = format!("ziputil-extraction/{}", zip_name);
            let dir_out = Path::new(&dirname);
            utility::extract_files(&zipfile, &to_take[..], dir_out)?;
        }
        Command::View => {
            let to_take = utility::choose_from_vector(&matches)?;
            utility::display_files(&zipfile, &to_take[..])?;
        }
    }

    Ok(())
}

#[derive(Debug)]
enum Command {
    Choose,
    View,
    List,
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
