// Create a .tour folder
// Populate with tour/steps/0/message
// Populate with tour/steps/0/files/file1
// Populate with tour/steps/0/files/file2

use crate::TOUR_DIR;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn init(files: Vec<PathBuf>, message: String) -> Result<(), std::io::Error> {
    if fs::exists(TOUR_DIR).is_ok() {
        panic!("{} folder exists", TOUR_DIR);
    }
    fs::create_dir_all(format!("{}/{}", TOUR_DIR, "steps/0/files"))?;

    let dest = format!("{}/steps/0/files/", TOUR_DIR);

    let mut message_file = File::create("./.tour/steps/0/message")?;
    write!(message_file, "{}", message)?;

    Ok(())
}
