use crate::error::CommitError;
use crate::utils::{is_descendant_of_current_dir, is_file_in_dir};
use crate::TOUR_DIR;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const STAGED_PATH: &str = "./.tour/staged";

pub fn add(files: Vec<PathBuf>) -> Result<(), CommitError> {
    let tour_dir = Path::new(TOUR_DIR);

    for file in &files {
        if !is_descendant_of_current_dir(file)? {
            return Err(CommitError::NotADescendantOfCurrentDir(file.clone()));
        }
        if is_file_in_dir(file, tour_dir)? {
            return Err(CommitError::InsideTourDir(file.clone()));
        }
    }

    let mut staged = OpenOptions::new()
        .append(true)
        .create(true)
        .open(STAGED_PATH)?;

    for file in &files {
        writeln!(staged, "{}", file.display())?;
        println!("staged: {}", file.display());
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
