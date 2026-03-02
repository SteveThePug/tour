// Create a .tour folder
// Create directory ./.tour/steps that stores tutorial steps
// Create file ./.tour/session that logs sessions information

use crate::TOUR_DIR;
use std::fs::DirBuilder;
use std::path::PathBuf;

// Creates the directory for tour
pub fn init() -> Result<(), std::io::Error> {
    let tour_dir = PathBuf::from(TOUR_DIR);

    DirBuilder::new()
        .recursive(true)
        .create(tour_dir.join("steps"))?;

    std::fs::File::create(tour_dir.join("session"))?;

    crate::info::set_info()
}
