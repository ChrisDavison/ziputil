use crate::utility;
use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Command {
    Choose,
    View,
    List,
    Extract,
}

impl Command {
    pub fn execute(&self, matches: &[String], zipfile: &PathBuf) -> Result<()> {
        match self {
            // List is a noop, as matches get printed during filtering
            Command::List => Command::display_matches(matches),
            Command::Extract => {
                let zip_name = zipfile.file_name()
                    .context("Couldn't get zip filename")?
                    .to_string_lossy()
                    .to_string();
                let dirname = format!("extracted_from_{}", zip_name);
                let dir_out = PathBuf::from(&dirname);
                utility::extract_files(zipfile, matches, &dir_out)
            },
            Command::Choose => {
                Command::display_matches(matches)?;
                let to_take = utility::choose_from_vector(matches)?;
                let zip_name = zipfile
                    .file_name()
                    .context("Couldn't get zip filename")?
                    .to_string_lossy()
                    .to_string();
                let dirname = format!("extracted_from_{}", zip_name);
                let dir_out = PathBuf::from(&dirname);
                utility::extract_files(zipfile, &to_take[..], &dir_out)
            }
            Command::View => {
                Command::display_matches(matches)?;
                let to_take = utility::choose_from_vector(matches)?;
                utility::display_files(zipfile, &to_take[..])
            }
        }
    }

    fn display_matches(matches: &[String]) -> Result<()> {
        if matches.is_empty() {
            return Ok(());
        }
        // println!("Matches");
        for (i, m) in matches.iter().enumerate() {
            println!("{}. {}", i, m);
        }
        Ok(())
    }
}
