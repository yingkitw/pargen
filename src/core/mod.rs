pub mod error;
pub mod traits;

pub use error::{Error, Result, Diagnostic, DiagnosticSeverity, ErrorBag, SourceLocation};
pub use traits::{GrammarParser, SemanticAnalyzer, CodeGenerator};
