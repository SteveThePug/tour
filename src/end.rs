use crate::error::TourError;
use crate::utils::{get_tour_step, require_tour};
use crate::TOUR_DIR;
use std::fs;
use std::path::Path;

pub fn end(message: String) -> Result<(), TourError> {
    require_tour()?;

    let end_marker = Path::new(TOUR_DIR).join("ended");
    if end_marker.exists() {
        return Err(TourError::TourEnded);
    }

    let step_count = get_tour_step()?;
    if step_count == 0 {
        return Err(TourError::NoSteps);
    }

    fs::write(&end_marker, &message)?;
    crate::info::update_last_modified()?;

    println!("Tour ended with {} steps: {}", step_count, message);
    Ok(())
}
