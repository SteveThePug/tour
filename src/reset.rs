use crate::error::TourError;
use crate::step;
use crate::utils::require_tour;
use crate::SESSION_PATH;
use std::fs;

pub fn reset() -> Result<(), TourError> {
    require_tour()?;

    let cwd = std::env::current_dir()?;
    let tracked = step::get_tracked_files()?;
    step::remove_tracked_files(&cwd, &tracked)?;

    let _ = fs::remove_file(SESSION_PATH);

    println!("Tour session reset. Tracked files removed from working directory.");
    Ok(())
}
