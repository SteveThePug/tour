use crate::TOUR_DIR;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub fn commit(files: Vec<PathBuf>, message: String) -> Result<(), std::io::Error> {
    // let files = files.iter().map(|p| p.as_ref()).collect();
    //
    // let dir = std::fs::read_dir(format!("{}/steps", TOUR_DIR))?;
    //
    // // USE /steps to find number of next step
    // // let step_number =
    //
    // fs::create_dir_all(format!("{}/{}", TOUR_DIR, "steps/0/files"))?;
    //
    // // Copy files listed by command to step 0
    // let dest = format!("{}/steps/0/files/", TOUR_DIR);
    // crate::utils::copy_files(files, dest.as_ref())?;
    //
    // // Copy message
    // let mut message_file = fs::File::create(format!("{}/steps/{}/message", TOUR_DIR, step_number))?;
    // write!(message_file, "{}", message)?;
    Ok(())
}
