use anyhow::{Context, Result};
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Filter {
    any: bool,
    ordered: bool,
    query: Vec<String>,
}

impl Filter {
    pub fn new(any: bool, ordered: bool, query: Vec<String>) -> Filter {
        Filter {
            any,
            ordered,
            query,
        }
    }

    pub fn matches(&self, string: &str) -> bool {
        if self.any {
            self.anymatch(string)
        } else {
            self.fuzzymatch(string)
        }
    }

    fn fuzzymatch(&self, string: &str) -> bool {
        let mut idx = 0;
        for word in &self.query {
            // If ordered search, search from the match of the
            // previous query forwards
            // otherwise just search from start to end.
            match string[idx..].find(word) {
                Some(i) => idx = if self.ordered { i } else { 0 },
                None => return false,
            }
        }
        true
    }

    fn anymatch(&self, string: &str) -> bool {
        for word in &self.query {
            if string.contains(word) {
                return true;
            }
        }
        false
    }
}

fn parse_range(s: &str) -> Vec<usize> {
    let start_and_end = s
        .split('-')
        .map(|x| x.parse().unwrap())
        .collect::<Vec<usize>>();
    let start = start_and_end[0];
    let end = start_and_end[1] + 1;
    (start..end).collect()
}

fn get_number_choices() -> Vec<usize> {
    let response = read_from_stdin("Choices: ");
    let mut nums = Vec::new();
    for num in response.trim().split(' ') {
        if num.contains('-') {
            nums.extend(parse_range(num));
        } else {
            nums.push(num.parse().unwrap())
        }
    }
    nums
}

fn read_from_stdin(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = stdout().flush();
    let mut response = String::new();
    stdin().read_line(&mut response).unwrap();
    response.trim().to_string()
}

fn extract_files(zipfile: &str, names: &[String], outdir: &Path) -> Result<()> {
    let f = File::open(&zipfile)?;
    let mut z = zip::ZipArchive::new(f)?;
    for name in names {
        let mut fmatch = z.by_name(name)?;
        let fullname = outdir.join(fmatch.name());
        let pp = fullname.parent().with_context(|| "No parent")?;
        println!("-- {:?}", fullname);
        if fmatch.is_dir() {
            create_dir_all(fullname)?;
        } else {
            create_dir_all(pp)?;
            let mut w = File::create(fullname)?;
            std::io::copy(&mut fmatch, &mut w)?;
        };
    }
    Ok(())
}

fn display_files(zipfile: &str, names: &[String]) -> Result<()> {
    let f = File::open(&zipfile)?;
    let mut z = zip::ZipArchive::new(f)?;
    for (i, name) in names.iter().enumerate() {
        println!("{}\n", &name);
        let fmatch = z.by_name(name)?;
        let mut bufr = std::io::BufReader::new(fmatch);
        std::io::copy(&mut bufr, &mut stdout())?;
        if i != names.len() {
            println!("{}", "-".repeat(20));
        }
    }
    Ok(())
}

fn get_matches(zipfile: &str, filter: Filter) -> Result<Option<Vec<String>>> {
    let f = File::open(&zipfile)?;
    let mut z = zip::ZipArchive::new(f)?;
    let mut matches = Vec::new();
    for i in 0..z.len() {
        let name = z.by_index(i).unwrap().name().to_string();
        if filter.matches(&name) {
            matches.push(name.to_string());
        }
    }
    if !matches.is_empty() {
        println!("Matches");
        for (i, m) in matches.iter().enumerate() {
            println!("{}. {}", i, m);
        }
        Ok(Some(matches))
    } else {
        Ok(None)
    }
}

fn choose_from_vector(vector: &[String]) -> Vec<String> {
    let choices = get_number_choices();
    let mut to_take = Vec::new();
    for choice in choices {
        to_take.push(vector[choice].to_string());
    }
    to_take
}

fn main() -> Result<()> {
    let (command, zipfile, filter) = parse_args()?;
    let matches = match get_matches(&zipfile, filter)? {
        Some(m) => m,
        None => {
            println!("No matching files in zipfile.");
            std::process::exit(0);
        }
    };

    match command {
        Command::List => display_files(&zipfile, &matches)?,
        Command::Choose => {
            let to_take = choose_from_vector(&matches);
            let zip_name = PathBuf::from(&zipfile)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let dirname = format!("ziputil-extraction/{}", zip_name);
            let dir_out = Path::new(&dirname);
            extract_files(&zipfile, &to_take[..], dir_out)?;
        }
        Command::View => {
            let to_take = choose_from_vector(&matches);
            display_files(&zipfile, &to_take[..])?;
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

const USAGE: &'static str = "usage: ziputil <command> <zipfile> [args] [query]...

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
