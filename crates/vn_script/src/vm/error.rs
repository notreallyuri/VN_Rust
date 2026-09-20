use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    UnknownScene(String),
    NoChoicePending,
    ChoiceOutOfRange { index: usize, options: usize },
    UnknownVariable(String),
    TypeMismatch { variable: String, message: String },
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::UnknownScene(id) => write!(f, "no scene named '{}'", id),
            VmError::NoChoicePending => write!(f, "no choice is waiting for an answer"),
            VmError::ChoiceOutOfRange { index, options } => {
                write!(f, "choice {} is out of range ({} options)", index, options)
            }
            VmError::UnknownVariable(id) => write!(f, "unknown variable '{}'", id),
            VmError::TypeMismatch { variable, message } => {
                write!(f, "variable '{}': {}", variable, message)
            }
        }
    }
}

impl std::error::Error for VmError {}
