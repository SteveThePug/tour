use crate::error::TourError;
use crate::step;
use crate::utils::require_tour;
use crate::SESSION_PATH;
use std::fs;
use std::io::{self, IsTerminal, Write};

pub fn reset(force: bool) -> Result<(), TourError> {
    require_tour()?;

    let cwd = std::env::current_dir()?;
    let tracked = step::get_tracked_files()?;
    let count = tracked.iter().filter(|f| cwd.join(f).is_file()).count();

    if !force && count > 0 {
        if !io::stdin().is_terminal() {
            return Err(TourError::ResetNeedsForce);
        }
        print!(
            "Remove {} tracked file{} from the working directory? [y/N] ",
            count,
            if count == 1 { "" } else { "s" }
        );
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim(), "y" | "Y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    }

    step::remove_tracked_files(&cwd, &tracked)?;

    let _ = fs::remove_file(SESSION_PATH);

    println!("Tour session reset. Tracked files removed from working directory.");
    Ok(())
}
