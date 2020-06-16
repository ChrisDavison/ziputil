use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::path::Path;
use zip;

use clap::Clap;

type Result<T> = std::result::Result<T, Box<dyn ::std::error::Error>>;

/// Choose zip files to operate on, based on a fuzzy query
#[derive(Clap)]
#[clap(version = "1.0", author = "Chris Davison <c.jr.davison@gmail.com>")]
struct Opts {
    /// The command to run (choose or view)
    command: String,
    /// The zip file to view files from
    zipfile: String,
    /// Word list to filter by
    query: Vec<String>,
}

fn fuzzy_contains(s: &str, query: &[String]) -> bool {
    let mut idx = 0;
    for word in query {
        // Search for each word from current position onwards
        // if any search fails, return false
        match s[idx..].find(word) {
            Some(i) => idx = i,
            None => return false,
        }
    }
    // If all sub-word searches complete, return true
    true
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

fn extract_files(
    z: &mut zip::ZipArchive<impl Read + Seek>,
    names: &[String],
    outdir: &Path,
) -> Result<()> {
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

fn display_files(z: &mut zip::ZipArchive<impl Read + Seek>, names: &[String]) -> Result<()> {
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

fn get_matches(zipfile: &std::fs::File, query: &[String]) -> Result<Vec<String>> {
    let mut z = zip::ZipArchive::new(zipfile)?;
    println!("Matches");
    let mut matches = Vec::new();
    let mut j = 0;
    for i in 0..z.len() {
        let name = z.by_index(i).unwrap().name().to_string();
        if fuzzy_contains(&name, query) {
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
    let opts: Opts = Opts::parse();
    let query = if opts.query.is_empty() {
        let resp = read_from_stdin("Query: ");
        resp.split(" ").map(|x| x.to_string()).collect()
    } else {
        opts.query
    };
    let f = File::open(&opts.zipfile)?;
    let matches = get_matches(&f, &query[..])?;
    let to_take = choose_from_vector(&matches);
    
    let mut z = zip::ZipArchive::new(f)?;
    match opts.command.as_str() {
        "choose" => {
            let dirname = format!("files-from-{}", opts.zipfile.replace(".", "-"));
            let dir_out = Path::new(&dirname);
            extract_files(&mut z, &to_take[..], &dir_out)?;
        }
        "view" => display_files(&mut z, &to_take[..])?,
        _ => println!("Unrecognised command: {}", opts.command),
    }

    Ok(())
}
