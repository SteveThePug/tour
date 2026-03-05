use crate::add::{clear_staged, get_staged};
use crate::error::TourError;
use crate::rm::{clear_removed, get_removed};
use crate::utils::{copy_path, get_tour_step, is_descendant_of_current_dir, is_file_in_dir, require_tour};
use crate::TOUR_DIR;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn commit(files: Vec<PathBuf>, message: String) -> Result<(), TourError> {
    require_tour()?;
    let tour_dir = Path::new(TOUR_DIR);

    if tour_dir.join("ended").exists() {
        return Err(TourError::TourEnded);
    }

    let used_staging = files.is_empty();
    let files = if used_staging {
        let staged = get_staged()?;
        if staged.is_empty() {
            return Err(TourError::NothingToCommit);
        }
        staged
    } else {
        files
    };

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

    let removed = get_removed()?;
    let removed_set: HashSet<PathBuf> = removed.into_iter().collect();

    let step_num = get_tour_step()? as usize;
    let steps_dir = tour_dir.join("steps");
    let step_dir = steps_dir.join(step_num.to_string());
    fs::create_dir_all(&step_dir)?;

    // Carry forward files from previous step (excluding removed files)
    if step_num > 0 {
        let prev_dir = steps_dir.join((step_num - 1).to_string());
        if prev_dir.exists() {
            carry_forward(&prev_dir, &prev_dir, &step_dir, &removed_set)?;
        }
    }

    // Overlay new files (these take precedence over carried-forward ones)
    for file in &files {
        copy_path(file, &step_dir)?;
    }

    fs::write(step_dir.join("message"), &message)?;

    // Only clear staging if we used it
    if used_staging {
        clear_staged()?;
    }
    clear_removed()?;
    crate::info::update_last_modified()?;

    println!("Step {}: {}", step_num + 1, message);
    Ok(())
}

fn carry_forward(
    step_root: &Path,
    src: &Path,
    dest_root: &Path,
    removed: &HashSet<PathBuf>,
) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let relative = entry
            .path()
            .strip_prefix(step_root)
            .unwrap_or(&entry.path())
            .to_path_buf();

        if relative == Path::new("message") {
            continue;
        }

        if removed.contains(&relative) {
            continue;
        }

        let dest = dest_root.join(&relative);
        if entry.path().is_dir() {
            fs::create_dir_all(&dest)?;
            carry_forward(step_root, &entry.path(), dest_root, removed)?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}
