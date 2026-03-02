use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;

mod commit;
mod end;
mod error;
mod init;
mod step;
mod utils;
mod info;

const TOUR_DIR: &str = "./.tour";
const SESSION_PATH: &str = "./.tour/session";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // Create a new tour
    Init,

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

    // Go to a specific step of tour
    Step {
        #[arg(value_name = "STEP")]
        n: i32,
    },

    // Go to beginning of tour
    Start,

    Info,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    match args.command {
        Some(Commands::Init) => crate::init::init()?,
        Some(Commands::Commit { files, message }) => crate::commit::commit(files, message)?,
        Some(Commands::End { message }) => crate::end::end(message)?,
        Some(Commands::Next { n }) => crate::step::next(n)?,
        Some(Commands::Prev { n }) => crate::step::prev(n)?,
        Some(Commands::Step { n }) => crate::step::step_n(n)?,
        Some(Commands::Start) => crate::step::step_n(0)?,
        Some(Commands::Info) => crate::info::info()?,
        _ => println!("command not found"),
    }
    Ok(())
}
