// Create a .tour folder
// Populate with tour/steps/0/message
// Populate with tour/steps/0/files/file1
// Populate with tour/steps/0/files/file2

use crate::TOUR_DIR;
use crate::utils::copy_files;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn init(files: Vec<PathBuf>, message: String) -> Result<(), std::io::Error> {
    // Convert PathBuf to &Path
    let files = files.iter().map(|p| p.as_path()).collect();

    // Check TOUR_DIR exists (it shouldn't because user calls init)
    if fs::exists(TOUR_DIR)? {
        panic!("{} folder exists", TOUR_DIR);
    }
    // Create dir recursively
    fs::create_dir_all(format!("{}/{}", TOUR_DIR, "steps/0/files"))?;

    // Copy files listed by command to step 0
    let dest = format!("{}/steps/0/files/", TOUR_DIR);
    copy_files(files, dest.as_ref())?;

    // Copy message to step 0
    let mut message_file = File::create("./.tour/steps/0/message")?;
    write!(message_file, "{}", message)?;

    Ok(())
}
