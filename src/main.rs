use std::env;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::path::Path;
use zip;

const USAGE: &str = "Usage: ziputil command zipfile [query...]

Commands:
    choose - extract files matching query
    view   - print chosen files to terminal
    list   - list matching files";

type Result<T> = std::result::Result<T, Box<dyn ::std::error::Error>>;

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
            if let Some(_) = string.find(word) {
                return true;
            }
        }
        false
    }
}

fn parse_range(s: &str) -> Vec<usize> {
    let start_and_end = s
        .split("-")
        .map(|x| x.parse().unwrap())
        .collect::<Vec<usize>>();
    let start = start_and_end[0];
    let end = start_and_end[1] + 1;
    (start..end).collect()
}

fn get_number_choices() -> Vec<usize> {
    let response = read_from_stdin("Choices: ");
    let mut nums = Vec::new();
    for num in response.trim().split(" ") {
        if num.contains("-") {
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
        let mut fmatch = z.by_name(&name)?;
        let fullname = outdir.join(fmatch.name());
        let pp = fullname.parent().ok_or("No parent")?;
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
        let fmatch = z.by_name(&name)?;
        let mut bufr = std::io::BufReader::new(fmatch);
        std::io::copy(&mut bufr, &mut stdout())?;
        if i != names.len() {
            println!("{}", "-".repeat(20));
        }
    }
    Ok(())
}

fn get_matches(zipfile: &str, filter: Filter) -> Result<Vec<String>> {
    let f = File::open(&zipfile)?;
    let mut z = zip::ZipArchive::new(f)?;
    println!("Matches");
    let mut matches = Vec::new();
    let mut j = 0;
    for i in 0..z.len() {
        let name = z.by_index(i).unwrap().name().to_string();
        if filter.matches(&name) {
            println!("{}. {}", j, name);
            j += 1;
        }
        matches.push(name.to_string());
    }
    Ok(matches)
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
    let args: Vec<String> = env::args().skip(1)
        .filter(|x| !x.starts_with('-')).collect();

    let flags: Vec<String> = env::args().skip(1)
        .filter(|x| x.starts_with('-')).collect();
    
    let wants_help = flags.contains(&"-h".to_string()) || flags.contains(&"--help".to_string()) ;
    if args.len() < 3 || wants_help {
        print!("{}", &USAGE);
        return Ok(());
    }

    let command = &args[1];
    let zipfile = &args[2];
    let query = args[3..].to_vec();

    let any = flags.contains(&"-a".to_string()) || flags.contains(&"--any".to_string()) ;
    let ordered = !flags.contains(&"-u".to_string()) || flags.contains(&"--unordered".to_string()) ;

    let filter = Filter::new(any, ordered, query);
    let matches = get_matches(&zipfile, filter)?;

    match command.as_str() {
        "choose" => {
            let to_take = choose_from_vector(&matches);
            let dirname = format!("files-from-{}", zipfile.replace(".", "-"));
            let dir_out = Path::new(&dirname);
            extract_files(&zipfile, &to_take[..], &dir_out)?;
        }
        "view" => {
            let to_take = choose_from_vector(&matches);
            display_files(&zipfile, &to_take[..])?;
        }
        "list" => {},
        _ => println!("Unrecognised command: {}", command),
    }

    Ok(())
}
