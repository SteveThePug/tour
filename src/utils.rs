use std::fs;
use std::io;
use std::path::Path;

use crate::SESSION_PATH;
use crate::TOUR_DIR;

/// Copies a file or directory into dest_dir, preserving relative path structure.
/// e.g. `src/main.rs` → `dest_dir/src/main.rs`
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

pub fn is_descendant_of_current_dir(file: &Path) -> Result<bool, io::Error> {
    is_file_in_dir(file, &std::env::current_dir()?)
}

pub fn is_file_in_dir(file: &Path, dir: &Path) -> Result<bool, io::Error> {
    let file_canon = file.canonicalize()?;
    let dir_canon = dir.canonicalize()?;
    Ok(file_canon.starts_with(&dir_canon))
}

pub fn get_session_step() -> Result<u32, Box<dyn std::error::Error>> {
    let session = fs::read_to_string(SESSION_PATH)?;
    let step = session
        .split("STEP=")
        .nth(1)
        .ok_or("no STEP in session")?
        .trim()
        .parse::<u32>()?;
    Ok(step)
}

pub fn get_tour_step() -> Result<u32, Box<dyn std::error::Error>> {
    let steps_dir = Path::new(TOUR_DIR).join("steps");
    let count = fs::read_dir(&steps_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .count() as u32;
    Ok(count)
}
