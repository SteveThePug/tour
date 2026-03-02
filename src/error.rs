use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CommitError {
    NotADescendantOfCurrentDir(PathBuf),
    InsideTourDir(PathBuf),
    Io(io::Error),
}

impl std::fmt::Display for CommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotADescendantOfCurrentDir(path) => {
                write!(f, "File {:?} is not a descendant of the working directory.", path)
            }
            Self::InsideTourDir(path) => {
                write!(f, "File {:?} is inside a .tour directory, which is not allowed.", path)
            }
            Self::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for CommitError {}

impl From<io::Error> for CommitError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
