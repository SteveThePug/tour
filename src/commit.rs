use crate::error::CommitError;
use crate::utils::{copy_path, is_descendant_of_current_dir, is_file_in_dir};
use crate::TOUR_DIR;
use std::fs;
use std::path::{Path, PathBuf};

pub fn commit(files: Vec<PathBuf>, message: String) -> Result<(), CommitError> {
    let tour_dir = Path::new(TOUR_DIR);

    for file in &files {
        if !is_descendant_of_current_dir(file)? {
            return Err(CommitError::NotADescendantOfCurrentDir(file.clone()));
        }
        if is_file_in_dir(file, tour_dir)? {
            return Err(CommitError::InsideTourDir(file.clone()));
        }
    }

    let steps_dir = tour_dir.join("steps");
    let step_num = fs::read_dir(&steps_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .count();

    let step_dir = steps_dir.join(step_num.to_string());
    fs::create_dir_all(&step_dir)?;

    for file in &files {
        copy_path(file, &step_dir)?;
    }

    fs::write(step_dir.join("message"), &message)?;
    crate::info::update_last_modified()?;

    println!("Step {}: {}", step_num, message);
    Ok(())
}
