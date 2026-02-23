use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;

mod commit;
mod end;
mod init;
mod next;
mod prev;
mod utils;

const TOUR_DIR: &str = "./.tour";
const DEFAULT_SESSION: &str = "./.tour/session";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // Create a new tour
    Init {
        #[arg(value_name = "FILES")]
        files: Vec<PathBuf>,
        #[arg(short, value_name = "MESSAGE")]
        message: String,
    },
    // Add steps to the tour
    Commit {
        files: Vec<PathBuf>,

        #[arg(short, long, value_name = "MESSAGE")]
        message: String,
    },
    // Finish the tour
    End {
        #[arg(short, long, value_name = "MESSAGE")]
        message: String,
    },

    // Go to next step of tour
    Next {
        #[arg(short, value_name = "NUM STEPS")]
        n: Option<i32>,
    },
    // Go to previous step of tour
    Prev {
        #[arg(short, value_name = "NUM STEPS")]
        n: Option<i32>,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    match args.command {
        Some(Commands::Init { files, message }) => crate::init::init(files, message)?,
        Some(Commands::Commit { files, message }) => crate::commit::commit(files, message)?,
        Some(Commands::End { message }) => crate::end::end(message)?,
        Some(Commands::Next { n }) => crate::next::next(n)?,
        Some(Commands::Prev { n }) => crate::prev::prev(n)?,
        _ => println!("command not found"),
    }
    Ok(())
}
