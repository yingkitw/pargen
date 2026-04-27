pub mod core;
pub mod grammar;
pub mod analysis;
pub mod codegen;
pub mod mcp;

pub use core::{Error, Result, Diagnostic, DiagnosticSeverity};
pub use core::{GrammarParser, SemanticAnalyzer, CodeGenerator};
pub use grammar::{Grammar, GrammarKind, Rule, RuleModifier, Alternative, Element, ElementKind};
pub use analysis::ProcessedGrammar;

use anyhow::Result as AnyhowResult;

pub fn parse_grammar_file(path: &str) -> AnyhowResult<grammar::Grammar> {
    let source = std::fs::read_to_string(path)?;
    parse_grammar_source(&source)
}

pub fn parse_grammar_source(source: &str) -> AnyhowResult<grammar::Grammar> {
    let lexer = grammar::G4Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let parser = grammar::G4Parser::new(tokens, source.to_string());
    let grammar = parser.parse()?;
    Ok(grammar)
}

pub fn generate(grammar: grammar::Grammar, lang: &str, output_dir: &str) -> AnyhowResult<()> {
    let processed = analysis::ProcessedGrammar::process(grammar)?;
    let output_path = std::path::Path::new(output_dir);
    codegen::generate_output(&processed, lang, output_path)?;
    Ok(())
}
