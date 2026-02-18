use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;

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
        #[arg(short, long, value_name = "MESSAGE")]
        message: Option<String>,
    },
    // Add steps to the tour
    Commit {
        files: Vec<PathBuf>,

        #[arg(short, long, value_name = "MESSAGE")]
        message: Option<String>,
    },
    // Finish the tour
    End {
        #[arg(short, long, value_name = "MESSAGE")]
        message: Option<String>,
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

    Ok(())
}
