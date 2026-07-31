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

    #[error("Multiple errors:\n{diagnostics}")]
    Multiple { diagnostics: ErrorBag },
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

    pub fn from_diagnostics(diagnostics: Vec<Diagnostic>) -> Self {
        Error::Multiple {
            diagnostics: ErrorBag::from(diagnostics),
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

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column, file: None }
    }

    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }
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

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticSeverity::Error => write!(f, "error"),
            DiagnosticSeverity::Warning => write!(f, "warning"),
            DiagnosticSeverity::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub location: Option<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            location: None,
            note: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            location: None,
            note: None,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Info,
            message: message.into(),
            location: None,
            note: None,
        }
    }

    pub fn at(mut self, line: usize, column: usize) -> Self {
        self.location = Some(SourceLocation::new(line, column));
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ErrorBag {
    diagnostics: Vec<Diagnostic>,
}

impl ErrorBag {
    pub fn new() -> Self {
        Self { diagnostics: Vec::new() }
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn push_error(&mut self, message: impl Into<String>) -> &mut Self {
        self.diagnostics.push(Diagnostic::error(message));
        self
    }

    pub fn push_warning(&mut self, message: impl Into<String>) -> &mut Self {
        self.diagnostics.push(Diagnostic::warning(message));
        self
    }

    pub fn push_lexer(
        &mut self,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> &mut Self {
        self.diagnostics.push(Diagnostic::error(message).at(line, column));
        self
    }

    pub fn push_parser(
        &mut self,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> &mut Self {
        self.diagnostics.push(Diagnostic::error(message).at(line, column));
        self
    }

    pub fn extend<I: IntoIterator<Item = Diagnostic>>(&mut self, iter: I) {
        self.diagnostics.extend(iter);
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Diagnostic> {
        self.diagnostics.iter()
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Warning)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count()
    }

    pub fn to_error(self) -> Option<Error> {
        if self.diagnostics.is_empty() {
            None
        } else if self.diagnostics.len() == 1 {
            // Collapse to a single Error to keep the simple API ergonomic.
            let d = self.diagnostics.into_iter().next().unwrap();
            match d.severity {
                DiagnosticSeverity::Error => {
                    let loc = d.location.unwrap_or(SourceLocation::new(0, 0));
                    Some(Error::Parser { location: loc, message: d.message })
                }
                _ => Some(Error::ValidationFailed { count: 1 }),
            }
        } else {
            Some(Error::Multiple { diagnostics: self })
        }
    }
}

impl From<Vec<Diagnostic>> for ErrorBag {
    fn from(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }
}

impl From<Diagnostic> for ErrorBag {
    fn from(diagnostic: Diagnostic) -> Self {
        Self { diagnostics: vec![diagnostic] }
    }
}

impl std::fmt::Display for ErrorBag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, d) in self.diagnostics.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", d)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.location {
            Some(loc) => write!(f, "{}: {}: {}", self.severity, loc, self.message),
            None => write!(f, "{}: {}", self.severity, self.message),
        }?;
        if let Some(note) = &self.note {
            write!(f, "\n  = note: {}", note)?;
        }
        Ok(())
    }
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
            note: None,
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
            note: None,
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

    #[test]
    fn test_diagnostic_builder_pattern() {
        let d = Diagnostic::error("bad token")
            .at(3, 4)
            .with_note("did you mean ';'?");
        assert_eq!(d.severity, DiagnosticSeverity::Error);
        assert_eq!(d.message, "bad token");
        assert_eq!(d.location.unwrap().line, 3);
        assert_eq!(d.note.unwrap(), "did you mean ';'?");
    }

    #[test]
    fn test_error_bag_push_and_collect() {
        let mut bag = ErrorBag::new();
        bag.push(Diagnostic::error("first").at(1, 1));
        bag.push(Diagnostic::warning("second").at(2, 2));
        bag.push_parser(3, 3, "third");
        assert_eq!(bag.len(), 3);
        assert!(bag.has_errors());
        assert!(bag.has_warnings());
        assert_eq!(bag.error_count(), 2);
    }

    #[test]
    fn test_error_bag_display() {
        let mut bag = ErrorBag::new();
        bag.push(Diagnostic::error("oops").at(1, 1));
        let s = bag.to_string();
        assert!(s.contains("oops"));
        assert!(s.contains("1:1"));
    }

    #[test]
    fn test_error_bag_to_error_collapses_single() {
        let mut bag = ErrorBag::new();
        bag.push(Diagnostic::error("only one").at(2, 3));
        let err = bag.to_error().unwrap();
        assert!(matches!(err, Error::Parser { .. }));
        assert!(err.to_string().contains("only one"));
    }

    #[test]
    fn test_error_bag_to_error_uses_multiple_variant() {
        let mut bag = ErrorBag::new();
        bag.push(Diagnostic::error("a").at(1, 1));
        bag.push(Diagnostic::error("b").at(2, 2));
        let err = bag.to_error().unwrap();
        assert!(matches!(err, Error::Multiple { .. }));
    }

    #[test]
    fn test_error_from_diagnostics() {
        let diagnostics = vec![
            Diagnostic::error("a").at(1, 1),
            Diagnostic::error("b").at(2, 2),
        ];
        let err = Error::from_diagnostics(diagnostics);
        assert!(matches!(err, Error::Multiple { .. }));
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(DiagnosticSeverity::Error.to_string(), "error");
        assert_eq!(DiagnosticSeverity::Warning.to_string(), "warning");
        assert_eq!(DiagnosticSeverity::Info.to_string(), "info");
    }
}
