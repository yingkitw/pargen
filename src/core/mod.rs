pub mod error;
pub mod traits;

pub use error::{Error, Result, Diagnostic, DiagnosticSeverity};
pub use traits::{GrammarParser, SemanticAnalyzer, CodeGenerator};
