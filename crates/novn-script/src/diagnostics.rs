use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub file: Option<String>,
    pub line: usize,
    pub message: String,
}

impl Diagnostic {
    pub fn error(line: usize, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            file: None,
            line,
            message: message.into(),
        }
    }

    pub fn warning(line: usize, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            file: None,
            line,
            message: message.into(),
        }
    }

    pub fn with_file(mut self, file: Option<&str>) -> Self {
        self.file = file.map(str::to_string);
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
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
        match (&self.file, self.line) {
            (Some(file), 0) => write!(f, "{}: ", file)?,
            (Some(file), line) => write!(f, "{}:{}: ", file, line)?,
            (None, 0) => {}
            (None, line) => write!(f, "line {}: ", line)?,
        }
        write!(f, "{}: {}", self.severity.label(), self.message)
    }
}
