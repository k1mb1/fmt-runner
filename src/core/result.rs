use std::path::PathBuf;

/// Severity levels for formatter diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

/// Diagnostic emitted during formatting.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub range: Option<(usize, usize)>,
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub source: Option<String>,
}

impl Diagnostic {
    pub fn engine_error(range: Option<(usize, usize)>, message: impl Into<String>) -> Self {
        Self {
            range,
            message: message.into(),
            severity: DiagnosticSeverity::Error,
            source: Some("engine".to_string()),
        }
    }
}

/// Outcome for a single formatted file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileFormatOutcome {
    pub path: Option<PathBuf>,
    pub changed: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub diff: Option<String>,
}

impl FileFormatOutcome {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            changed: false,
            diagnostics: Vec::new(),
            diff: None,
        }
    }
}
