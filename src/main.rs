use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod add;
mod commit;
mod end;
mod error;
mod info;
mod init;
mod list;
mod reset;
mod rm;
mod status;
mod step;
mod unstage;
mod utils;

const TOUR_DIR: &str = "./.tour";
const SESSION_PATH: &str = "./.tour/session";

#[derive(Parser)]
#[command(author, version, about = "Create and navigate code tutorials as a series of snapshots", arg_required_else_help = true)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Set up a new tour in the current directory
    Init,

    /// Stage files for the next commit
    Add {
        files: Vec<PathBuf>,
    },

    /// Remove files from staging
    Unstage {
        files: Vec<PathBuf>,
    },

    /// Commit staged files as a new step
    Commit {
        files: Vec<PathBuf>,

        #[arg(short, long, value_name = "MESSAGE")]
        message: String,
    },

    /// Mark files for removal in the next commit
    Rm {
        files: Vec<PathBuf>,
    },

    /// Finalise the tour
    End {
        #[arg(short, long, value_name = "MESSAGE")]
        message: String,
    },

    /// Advance n steps (default 1)
    Next {
        #[arg(short, value_name = "NUM STEPS")]
        n: Option<u32>,
    },

    /// Go back n steps (default 1)
    Prev {
        #[arg(short, value_name = "NUM STEPS")]
        n: Option<u32>,
    },

    /// Jump to step n
    Step {
        #[arg(value_name = "STEP")]
        n: u32,
    },

    /// Load the first step
    Start,

    /// Show tour metadata
    Info,

    /// Show current step and staged files
    Status,

    /// List all steps with messages
    List,

    /// Reset tour session and remove tracked files
    Reset,
}

fn main() {
    let args = Args::parse();
    let result = match args.command {
        Some(Commands::Init) => crate::init::init(),
        Some(Commands::Add { files }) => crate::add::add(files),
        Some(Commands::Unstage { files }) => crate::unstage::unstage(files),
        Some(Commands::Commit { files, message }) => crate::commit::commit(files, message),
        Some(Commands::Rm { files }) => crate::rm::rm(files),
        Some(Commands::End { message }) => crate::end::end(message),
        Some(Commands::Next { n }) => crate::step::next(n),
        Some(Commands::Prev { n }) => crate::step::prev(n),
        Some(Commands::Step { n }) => crate::step::step_n(n),
        Some(Commands::Start) => crate::step::step_n(1),
        Some(Commands::Info) => crate::info::info(),
        Some(Commands::Status) => crate::status::status(),
        Some(Commands::List) => crate::list::list(),
        Some(Commands::Reset) => crate::reset::reset(),
        None => Ok(()),
    };
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
