use std::fmt;
use std::io;
use std::path::PathBuf;

use vn_script::Diagnostic;

#[derive(Debug)]
pub enum AppError {
    Story {
        path: PathBuf,
        source: io::Error,
    },
    Script {
        path: PathBuf,
        errors: Vec<Diagnostic>,
    },
    Schema {
        path: PathBuf,
        source: io::Error,
    },
    Screen(io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Story { source, .. } => write!(f, "could not load the story: {}", source),
            AppError::Script { path, errors } => {
                let count = errors.len();
                write!(
                    f,
                    "{} has {} error{}:",
                    path.display(),
                    count,
                    if count == 1 { "" } else { "s" }
                )?;
                for error in errors {
                    write!(f, "\n  {}", error)?;
                }
                Ok(())
            }
            AppError::Schema { source, .. } => {
                write!(f, "could not write the schema: {}", source)
            }
            AppError::Screen(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for AppError {}
