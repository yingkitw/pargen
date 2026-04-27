use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Lexer error at {location}: {message}")]
    Lexer { location: SourceLocation, message: String },

    #[error("Parser error at {location}: {message}")]
    Parser { location: SourceLocation, message: String },

    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("Code generation error: {0}")]
    Codegen(String),

    #[error("Unsupported language: {0}. Supported: rust, go, typescript, python, java, c, cpp")]
    UnsupportedLanguage(String),

    #[error("Validation failed with {count} diagnostic(s)")]
    ValidationFailed { count: usize },
}

impl Error {
    pub fn lexer(line: usize, column: usize, message: impl Into<String>) -> Self {
        Error::Lexer {
            location: SourceLocation { line, column, file: None },
            message: message.into(),
        }
    }

    pub fn parser(line: usize, column: usize, message: impl Into<String>) -> Self {
        Error::Parser {
            location: SourceLocation { line, column, file: None },
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.file {
            Some(file) => write!(f, "{}:{}:{}", file.display(), self.line, self.column),
            None => write!(f, "{}:{}", self.line, self.column),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub location: Option<SourceLocation>,
}
