use std::fs;
use std::io;
use std::path::Path;

use crate::error::TourError;
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
            s.split("STEP=")
                .nth(1)
                .and_then(|v| v.trim().parse::<u32>().ok())
        })
}

pub fn get_tour_step() -> Result<u32, TourError> {
    let steps_dir = Path::new(TOUR_DIR).join("steps");
    let mut indices: Vec<u32> = fs::read_dir(&steps_dir)?
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
        src.strip_prefix(&cwd).unwrap_or(src).to_path_buf()
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
        fs::copy(src, &dest)?;
    }
    Ok(())
}

/// Recursively copies src to dest. If src is a directory, copies its contents
/// into dest. If src is a file, copies it to dest.
pub fn copy_tree(src: &Path, dest: &Path) -> Result<(), io::Error> {
    if src.is_dir() {
        fs::create_dir_all(dest)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            copy_tree(&entry.path(), &dest.join(entry.file_name()))?;
        }
    } else {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dest)?;
    }
    Ok(())
}

pub fn is_descendant_of_current_dir(file: &Path) -> Result<bool, io::Error> {
    is_file_in_dir(file, &std::env::current_dir()?)
}

pub fn is_file_in_dir(file: &Path, dir: &Path) -> Result<bool, io::Error> {
    let file_canon = file.canonicalize()?;
    let dir_canon = dir.canonicalize()?;
    Ok(file_canon.starts_with(&dir_canon))
}
