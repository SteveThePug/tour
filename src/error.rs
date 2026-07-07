use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum TourError {
    NoTour,
    TourAlreadyExists,
    TourEnded,
    NothingToCommit,
    NoSteps,
    NotADescendant(PathBuf),
    InsideTourDir(PathBuf),
    FileNotFound(PathBuf),
    StepOutOfRange { step: u32, total: u32 },
    NoStepsToNavigate,
    NotStarted,
    ResetNeedsForce,
    CorruptedTour(String),
    Io(io::Error),
    IoContext { context: String, source: io::Error },
}

impl std::fmt::Display for TourError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoTour => {
                write!(f, "No tour found in this directory. Run `tour init` first.")
            }
            Self::TourAlreadyExists => write!(f, "A tour already exists in this directory."),
            Self::TourEnded => write!(f, "Tour has already been ended."),
            Self::NothingToCommit => {
                write!(f, "Nothing to commit. Use `tour add <files>` to stage files first.")
            }
            Self::NoSteps => {
                write!(f, "Cannot end a tour with no steps. Use `tour commit` to add steps first.")
            }
            Self::NotADescendant(p) => {
                write!(f, "File {} is not a descendant of the working directory.", p.display())
            }
            Self::InsideTourDir(p) => {
                write!(f, "File {} is inside a .tour directory, which is not allowed.", p.display())
            }
            Self::FileNotFound(p) => write!(f, "File not found: {}", p.display()),
            Self::StepOutOfRange { step, total } => {
                write!(f, "Step {} is out of range (1-{}).", step, total)
            }
            Self::NoStepsToNavigate => {
                write!(f, "This tour has no steps yet. Use `tour commit` to create one.")
            }
            Self::NotStarted => {
                write!(f, "Tour not started. Run `tour start` first.")
            }
            Self::ResetNeedsForce => {
                write!(f, "Refusing to reset without confirmation. Re-run with `--force`.")
            }
            Self::CorruptedTour(msg) => write!(f, "Tour data is corrupted: {}", msg),
            Self::Io(e) => write!(f, "{}", e),
            Self::IoContext { context, source } => write!(f, "{}: {}", context, source),
        }
    }
}

/// Attaches a human-readable operation description to an io::Error.
pub trait IoResultExt<T> {
    fn context(self, ctx: impl Into<String>) -> Result<T, TourError>;
}

impl<T> IoResultExt<T> for Result<T, io::Error> {
    fn context(self, ctx: impl Into<String>) -> Result<T, TourError> {
        self.map_err(|e| TourError::IoContext {
            context: ctx.into(),
            source: e,
        })
    }
}

impl std::error::Error for TourError {}

impl From<io::Error> for TourError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
