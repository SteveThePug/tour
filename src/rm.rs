use crate::error::TourError;
use crate::utils::require_tour;
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub const REMOVED_PATH: &str = "./.tour/removed";

pub fn rm(files: Vec<PathBuf>) -> Result<(), TourError> {
    require_tour()?;

    let existing = get_removed()?;
    let existing_set: HashSet<PathBuf> = existing.into_iter().collect();

    let mut removed_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(REMOVED_PATH)?;

    for file in &files {
        if existing_set.contains(file) {
            println!("already marked for removal: {}", file.display());
        } else {
            writeln!(removed_file, "{}", file.display())?;
            println!("marked for removal: {}", file.display());
        }
    }

    Ok(())
}

pub fn get_removed() -> Result<Vec<PathBuf>, std::io::Error> {
    let content = fs::read_to_string(REMOVED_PATH).unwrap_or_default();
    Ok(content
        .lines()
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .collect())
}

pub fn clear_removed() -> Result<(), std::io::Error> {
    fs::write(REMOVED_PATH, "")
}
