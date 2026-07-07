use crate::error::TourError;
use crate::style::{bold, reset};
use crate::utils::{get_current_step, get_tour_step, require_tour};
use crate::TOUR_DIR;
use std::fs;
use std::path::Path;

pub fn list() -> Result<(), TourError> {
    require_tour()?;
    let total = get_tour_step()?;
    let current = get_current_step();

    if total == 0 {
        println!("No steps yet.");
        return Ok(());
    }

    for i in 0..total {
        let step_dir = Path::new(TOUR_DIR).join("steps").join(i.to_string());
        let message = fs::read_to_string(step_dir.join("message"))
            .unwrap_or_else(|_| "(no message)".into());
        if current == Some(i) {
            println!("{}> {}. {}{}", bold(), i + 1, message.trim(), reset());
        } else {
            println!("  {}. {}", i + 1, message.trim());
        }
    }
    Ok(())
}
