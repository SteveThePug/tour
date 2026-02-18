use std::fs;
use std::io;
use std::path::Path;

pub fn copy_files(files: Vec<&Path>, dest_dir: &Path) -> Result<(), io::Error> {
    for file in files {
        // Get the relative path components
        let dest_path = dest_dir.join(file);

        // Create parent directories if they don't exist
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy the file
        fs::copy(file, dest_path)?;
    }
    Ok(())
}
