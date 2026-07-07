use crate::add::{clear_staged, get_staged};
use crate::error::TourError;
use crate::rm::{clear_removed, get_removed};
use crate::style::{bold, reset};
use crate::utils::{copy_path, get_tour_step, require_tour, validate_paths};
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

    validate_paths(&files)?;

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

    println!("{}committed step {}{}: {}", bold(), step_num + 1, reset(), message);
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
            // Unchanged files are hardlinked so a step costs only its changes;
            // fall back to a copy on filesystems without hardlink support.
            if fs::hard_link(entry.path(), &dest).is_err() {
                fs::copy(entry.path(), &dest)?;
            }
        }
    }
    Ok(())
}
