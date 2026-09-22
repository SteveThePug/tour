use crate::add::{get_staged, STAGED_PATH};
use crate::error::TourError;
use crate::utils::{normalize_path, require_active};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub fn unstage(files: Vec<PathBuf>) -> Result<(), TourError> {
    require_active()?;
    let staged = get_staged()?;
    let files: Vec<PathBuf> = files.iter().map(|f| normalize_path(f)).collect();
    let to_remove: HashSet<&PathBuf> = files.iter().collect();

    let remaining: Vec<&PathBuf> = staged.iter().filter(|f| !to_remove.contains(f)).collect();
    let content: String = remaining.iter().map(|f| format!("{}\n", f.display())).collect();
    fs::write(STAGED_PATH, content)?;

    for file in &files {
        if staged.contains(file) {
            println!("unstaged: {}", file.display());
        } else {
            println!("not staged: {}", file.display());
        }
    }
    Ok(())
}
