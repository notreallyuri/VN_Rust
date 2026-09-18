use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub line: usize,
    pub message: String,
}

impl Diagnostic {
    pub fn error(line: usize, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            line,
            message: message.into(),
        }
    }

    pub fn warning(line: usize, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            line,
            message: message.into(),
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    pub fn in_file(&self, path: impl fmt::Display) -> String {
        format!(
            "{}:{}: {}: {}",
            path,
            self.line,
            self.severity.label(),
            self.message
        )
    }
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}: {}: {}",
            self.line,
            self.severity.label(),
            self.message
        )
    }
}
