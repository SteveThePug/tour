use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;

mod add;
mod commit;
mod end;
mod error;
mod info;
mod init;
mod step;
mod utils;

const TOUR_DIR: &str = "./.tour";
const SESSION_PATH: &str = "./.tour/session";

#[derive(Parser)]
#[command(author, version, about, long_about = None, disable_help_subcommand = true)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // Create a new tour
    Init,

    // Stage files for the next commit
    Add {
        files: Vec<PathBuf>,
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

    // Go to a specific step of tour
    Step {
        #[arg(value_name = "STEP")]
        n: i32,
    },

    // Go to beginning of tour
    Start,

    Info,

    // Show help
    Help,
}

fn help() {
    println!(
        "\
\x1b[1mtour\x1b[0m — create and navigate code tutorials as a series of snapshots

\x1b[1mAUTHOR WORKFLOW\x1b[0m
  tour init                          Set up a new tour in the current directory
  tour add <files...>                Stage files for the next commit
  tour commit [-m <msg>]             Commit staged files as a new step
  tour commit <files...> -m <msg>    Stage and commit files in one step
  tour end -m <msg>                  Finalise the tour

\x1b[1mREADER WORKFLOW\x1b[0m
  tour start                         Load the first step
  tour next [n]                      Advance n steps (default 1)
  tour prev [n]                      Go back n steps (default 1)
  tour step <n>                      Jump to step n

\x1b[1mOTHER\x1b[0m
  tour info                          Show tour metadata
  tour help                          Show this help message"
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    match args.command {
        Some(Commands::Init) => crate::init::init()?,
        Some(Commands::Add { files }) => crate::add::add(files)?,
        Some(Commands::Commit { files, message }) => crate::commit::commit(files, message)?,
        Some(Commands::End { message }) => crate::end::end(message)?,
        Some(Commands::Next { n }) => crate::step::next(n)?,
        Some(Commands::Prev { n }) => crate::step::prev(n)?,
        Some(Commands::Step { n }) => crate::step::step_n(n)?,
        Some(Commands::Start) => crate::step::step_n(0)?,
        Some(Commands::Info) => crate::info::info()?,
        Some(Commands::Help) | None => help(),
    }
    Ok(())
}
