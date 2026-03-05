use crate::error::TourError;
use crate::TOUR_DIR;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn init() -> Result<(), TourError> {
    let tour_dir = PathBuf::from(TOUR_DIR);

    if tour_dir.exists() {
        return Err(TourError::TourAlreadyExists);
    }

    DirBuilder::new()
        .recursive(true)
        .create(tour_dir.join("steps"))?;

    fs::File::create(tour_dir.join("session"))?;

    crate::info::set_info()?;
    update_gitignore()?;

    Ok(())
}

fn update_gitignore() -> Result<(), std::io::Error> {
    let gitignore = Path::new(".gitignore");
    let entries = [".tour/session", ".tour/staged", ".tour/removed"];

    let existing = fs::read_to_string(gitignore).unwrap_or_default();
    let mut to_add = Vec::new();
    for entry in &entries {
        if !existing.lines().any(|l| l.trim() == *entry) {
            to_add.push(*entry);
        }
    }

    if !to_add.is_empty() {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(gitignore)?;
        if !existing.is_empty() && !existing.ends_with('\n') {
            writeln!(file)?;
        }
        for entry in to_add {
            writeln!(file, "{}", entry)?;
        }
    }
    Ok(())
}
