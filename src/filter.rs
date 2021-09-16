use std::fs::File;

use anyhow::Result;

#[derive(Debug)]
pub struct Filter {
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
            // previous query forwards (i..).
            // Otherwise just search from 0..end.
            match string[idx..].find(word) {
                Some(i) => idx = if self.ordered { i } else { 0 },
                None => return false,
            }
        }
        // Early exit occurs if any word DIDN'T match, so if we've got here
        // all words have matched
        true
    }

    fn anymatch(&self, string: &str) -> bool {
        for word in &self.query {
            if string.contains(word) {
                return true;
            }
        }
        // Early exit occurs if any word DID match, so if we've got here,
        // Something hasn't matched
        false
    }

    pub fn filter_zip_by_name(&self, zipfile: &str) -> Result<Option<Vec<String>>> {
        let f = File::open(&zipfile)?;
        let mut z = zip::ZipArchive::new(f)?;
        let mut matches = Vec::new();
        for i in 0..z.len() {
            let entry = z.by_index(i)?;
            let name = entry.name().to_string();
            if self.matches(&name) {
                matches.push(name);
            }
        }
        if matches.is_empty() {
            Ok(None)
        } else {
            Ok(Some(matches))
        }
    }
}
