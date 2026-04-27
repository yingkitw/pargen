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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_error_display() {
        let err = Error::lexer(5, 10, "unexpected character");
        assert_eq!(err.to_string(), "Lexer error at 5:10: unexpected character");
    }

    #[test]
    fn test_parser_error_display() {
        let err = Error::parser(3, 7, "expected ';'");
        assert_eq!(err.to_string(), "Parser error at 3:7: expected ';'");
    }

    #[test]
    fn test_io_error_from_std() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = Error::from(io_err);
        assert!(matches!(err, Error::Io(_)));
        assert!(err.to_string().contains("file missing"));
    }

    #[test]
    fn test_unsupported_language_error() {
        let err = Error::UnsupportedLanguage("swift".to_string());
        assert_eq!(err.to_string(), "Unsupported language: swift. Supported: rust, go, typescript, python, java, c, cpp");
    }

    #[test]
    fn test_analysis_error_display() {
        let err = Error::Analysis("undefined rule 'foo'".to_string());
        assert_eq!(err.to_string(), "Analysis error: undefined rule 'foo'");
    }

    #[test]
    fn test_codegen_error_display() {
        let err = Error::Codegen("failed to generate token enum".to_string());
        assert_eq!(err.to_string(), "Code generation error: failed to generate token enum");
    }

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation { line: 5, column: 10, file: None };
        assert_eq!(loc.to_string(), "5:10");
    }

    #[test]
    fn test_source_location_with_file() {
        let loc = SourceLocation { line: 1, column: 3, file: Some(PathBuf::from("grammar.g4")) };
        assert_eq!(loc.to_string(), "grammar.g4:1:3");
    }

    #[test]
    fn test_diagnostic_serialization() {
        let diag = Diagnostic {
            severity: DiagnosticSeverity::Error,
            message: "undefined rule".to_string(),
            location: Some(SourceLocation { line: 1, column: 1, file: None }),
        };
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("undefined rule"));
        assert!(json.contains("Error"));
    }

    #[test]
    fn test_diagnostic_deserialization() {
        let json = r#"{"severity":"Error","message":"test error","location":{"line":2,"column":3}}"#;
        let diag: Diagnostic = serde_json::from_str(json).unwrap();
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.message, "test error");
        assert_eq!(diag.location.as_ref().unwrap().line, 2);
    }

    #[test]
    fn test_diagnostic_without_location() {
        let diag = Diagnostic {
            severity: DiagnosticSeverity::Warning,
            message: "unused import".to_string(),
            location: None,
        };
        let json = serde_json::to_string(&diag).unwrap();
        assert!(json.contains("unused import"));
    }

    #[test]
    fn test_error_clone_equality() {
        let err = Error::Analysis("test".to_string());
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn test_source_location_clone_equality() {
        let loc = SourceLocation { line: 1, column: 2, file: None };
        let cloned = loc.clone();
        assert_eq!(loc, cloned);
    }
}
