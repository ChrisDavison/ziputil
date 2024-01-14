use anyhow::{anyhow, Context, Result};
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::path::Path;

fn parse_range(s: &str) -> Result<Vec<usize>> {
    let parts = s.split('-').collect::<Vec<&str>>();
    if parts.len() != 2 {
        return Err(anyhow!("Range must be A-B"));
    }
    let start = parts[0].parse()?;
    let end = parts[1].parse().map(|x: usize| x + 1)?;
    Ok((start..end).collect())
}

fn get_number_choices() -> Result<Vec<usize>> {
    let response = read_from_stdin("Choices: ")?;
    let mut nums = Vec::new();
    for num in response.trim().split(' ') {
        if num.contains('-') {
            nums.extend(parse_range(num)?);
        } else {
            nums.push(num.parse()?)
        }
    }
    Ok(nums)
}

fn read_from_stdin(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    let _ = stdout().flush();
    let mut response = String::new();
    stdin().read_line(&mut response)?;
    Ok(response.trim().to_string())
}

pub fn extract_files(zipfile: &str, names: &[String], outdir: &Path) -> Result<()> {
    let f = File::open(zipfile)?;
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

pub fn display_files(zipfile: &str, names: &[String]) -> Result<()> {
    let f = File::open(zipfile)?;
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

pub fn choose_from_vector(vector: &[String]) -> Result<Vec<String>> {
    let choices = get_number_choices()?;
    let mut to_take = Vec::new();
    for choice in choices {
        to_take.push(vector[choice].to_string());
    }
    Ok(to_take)
}
