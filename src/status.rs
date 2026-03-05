use crate::add::get_staged;
use crate::error::TourError;
use crate::utils::{get_current_step, get_tour_step, require_tour};
use crate::TOUR_DIR;
use std::path::Path;

pub fn status() -> Result<(), TourError> {
    require_tour()?;

    let total = get_tour_step()?;
    let ended = Path::new(TOUR_DIR).join("ended").exists();
    let current = get_current_step();

    println!("Tour: {} steps{}", total, if ended { " (ended)" } else { "" });

    match current {
        Some(step) => println!("Current step: {}/{}", step + 1, total),
        None => println!("Current step: not started"),
    }

    let staged = get_staged()?;
    if staged.is_empty() {
        println!("Staged files: none");
    } else {
        println!("Staged files:");
        for file in &staged {
            println!("  {}", file.display());
        }
    }

    Ok(())
}
