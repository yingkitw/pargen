pub mod core;
pub mod grammar;
pub mod analysis;
pub mod codegen;
pub mod mcp;

pub use core::{Error, Result, Diagnostic, DiagnosticSeverity, ErrorBag, SourceLocation};
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
    let parser = grammar::G4Parser::new(tokens);
    let grammar = parser.parse()?;
    Ok(grammar)
}

/// Parse a grammar source and return the partial grammar plus all collected
/// diagnostics, rather than bailing on the first error. The grammar may be
/// `None` if recovery failed completely; otherwise it reflects the rules
/// that were successfully parsed.
pub fn parse_grammar_source_detailed(source: &str) -> (Option<grammar::Grammar>, Vec<Diagnostic>) {
    let (tokens, lexer_diags) = grammar::G4Lexer::new(source).tokenize_recover();
    let (grammar, parser_diags) = grammar::G4Parser::new(tokens).parse_recover();
    let mut all = lexer_diags;
    all.extend(parser_diags);
    (grammar, all)
}

pub fn generate(grammar: grammar::Grammar, lang: &str, output_dir: &str) -> AnyhowResult<()> {
    let processed = analysis::ProcessedGrammar::process(grammar)?;
    let output_path = std::path::Path::new(output_dir);
    codegen::generate_output(&processed, lang, output_path)?;
    Ok(())
}
