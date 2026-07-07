use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::{IoResultExt, TourError};
use crate::SESSION_PATH;
use crate::TOUR_DIR;

pub fn require_tour() -> Result<(), TourError> {
    if !Path::new(TOUR_DIR).exists() {
        return Err(TourError::NoTour);
    }
    Ok(())
}

pub fn get_current_step() -> Option<u32> {
    fs::read_to_string(SESSION_PATH)
        .ok()
        .and_then(|s| {
            s.lines()
                .find_map(|l| l.strip_prefix("STEP="))
                .and_then(|v| v.trim().parse::<u32>().ok())
        })
}

pub fn get_tour_step() -> Result<u32, TourError> {
    let steps_dir = Path::new(TOUR_DIR).join("steps");
    let mut indices: Vec<u32> = fs::read_dir(&steps_dir)
        .context(format!("failed to read steps directory {}", steps_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str()?.parse::<u32>().ok())
        .collect();

    if indices.is_empty() {
        return Ok(0);
    }

    indices.sort();
    let count = indices.len() as u32;

    for (i, &idx) in indices.iter().enumerate() {
        if idx != i as u32 {
            return Err(TourError::CorruptedTour(
                format!("step directories are not sequential (expected {}, found {})", i, idx),
            ));
        }
    }

    Ok(count)
}

/// Copies a file or directory into dest_dir, preserving relative path structure.
pub fn copy_path(src: &Path, dest_dir: &Path) -> Result<(), io::Error> {
    let relative_src = if src.is_absolute() {
        let cwd = std::env::current_dir()?;
        src.strip_prefix(&cwd)
            .map_err(|_| io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("path '{}' is not under the current directory", src.display()),
            ))?
            .to_path_buf()
    } else {
        src.to_path_buf()
    };

    if src.is_dir() {
        let dest = dest_dir.join(&relative_src);
        fs::create_dir_all(&dest)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            copy_path(&entry.path(), dest_dir)?;
        }
    } else {
        let dest = dest_dir.join(&relative_src);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        // Unlink first: the destination may be hardlinked into a previous
        // step's snapshot, and copying in place would write through the link.
        if dest.exists() {
            fs::remove_file(&dest)?;
        }
        fs::copy(src, &dest)?;
    }
    Ok(())
}

/// Validates that every path exists, lives under the working directory, and is
/// not inside the .tour directory. Canonicalizes the anchors once, not per file.
pub fn validate_paths(files: &[PathBuf]) -> Result<(), TourError> {
    let cwd = std::env::current_dir()?.canonicalize()?;
    let tour_canon = Path::new(TOUR_DIR).canonicalize()?;
    for file in files {
        if !file.exists() {
            return Err(TourError::FileNotFound(file.clone()));
        }
        let canon = file.canonicalize()?;
        if !canon.starts_with(&cwd) {
            return Err(TourError::NotADescendant(file.clone()));
        }
        if canon.starts_with(&tour_canon) {
            return Err(TourError::InsideTourDir(file.clone()));
        }
    }
    Ok(())
}
