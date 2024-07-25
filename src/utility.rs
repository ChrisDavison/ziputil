use anyhow::{Context, Result};
use std::fs::{create_dir_all, File};
use std::io::stdout;
use std::path::Path;
use skim::prelude::*;
use std::io::Cursor;

pub fn extract_files(zipfile: &std::path::PathBuf, names: &[String], outdir: &Path) -> Result<()> {
    let f = File::open(zipfile)?;
    let mut z = zip::ZipArchive::new(f)?;
    for name in names {
        let mut fmatch = z.by_name(name)?;
        let fullname = outdir.join(fmatch.name());
        let pp = fullname.parent().with_context(|| "No parent")?;
        println!("{:?}", fullname);
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

pub fn display_files(zipfile: &std::path::PathBuf, names: &[String]) -> Result<()> {
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
    let options = SkimOptionsBuilder::default()
        .multi(true)
        .prompt(Some("File: "))
        .select1(true)
        .exit0(true)
        .bind(vec!["<C-c>:abort"])
        .build().unwrap();
    let itemreader = SkimItemReader::default();
    let input = vector.join("\n");
    let items = itemreader.of_bufread(Cursor::new(input));
    let selected = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_default();

    Ok(selected.iter().map(|x| x.output().to_string()).collect())
}
