use crate::error::TourError;
use crate::style::{green, reset};
use crate::utils::{require_tour, validate_paths};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub const STAGED_PATH: &str = "./.tour/staged";

pub fn add(files: Vec<PathBuf>) -> Result<(), TourError> {
    require_tour()?;
    validate_paths(&files)?;

    let existing = get_staged()?;
    let existing_set: std::collections::HashSet<PathBuf> = existing.into_iter().collect();

    let mut staged = OpenOptions::new()
        .append(true)
        .create(true)
        .open(STAGED_PATH)?;

    for file in &files {
        let normalized: PathBuf = file.components().collect();
        if existing_set.contains(&normalized) {
            println!("already staged: {}", normalized.display());
        } else {
            writeln!(staged, "{}", normalized.display())?;
            println!("{}staged:{} {}", green(), reset(), normalized.display());
        }
    }

    Ok(())
}

pub fn get_staged() -> Result<Vec<PathBuf>, std::io::Error> {
    let content = fs::read_to_string(STAGED_PATH).unwrap_or_default();
    Ok(content
        .lines()
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .collect())
}

pub fn clear_staged() -> Result<(), std::io::Error> {
    fs::write(STAGED_PATH, "")
}
