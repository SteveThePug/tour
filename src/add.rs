use crate::error::TourError;
use crate::utils::{is_descendant_of_current_dir, is_file_in_dir, require_tour};
use crate::TOUR_DIR;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const STAGED_PATH: &str = "./.tour/staged";

pub fn add(files: Vec<PathBuf>) -> Result<(), TourError> {
    require_tour()?;
    let tour_dir = Path::new(TOUR_DIR);

    for file in &files {
        if !file.exists() {
            return Err(TourError::FileNotFound(file.clone()));
        }
        if !is_descendant_of_current_dir(file)? {
            return Err(TourError::NotADescendant(file.clone()));
        }
        if is_file_in_dir(file, tour_dir)? {
            return Err(TourError::InsideTourDir(file.clone()));
        }
    }

    let existing = get_staged()?;
    let existing_set: std::collections::HashSet<PathBuf> = existing.into_iter().collect();

    let mut staged = OpenOptions::new()
        .append(true)
        .create(true)
        .open(STAGED_PATH)?;

    for file in &files {
        if existing_set.contains(file) {
            println!("already staged: {}", file.display());
        } else {
            writeln!(staged, "{}", file.display())?;
            println!("staged: {}", file.display());
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
