//! Smoke tests for the criterion bench surface.
//!
//! Criterion benches in `benches/parser_codegen.rs` measure timing of the
//! parser + codegen pipeline. They're tested in `cargo bench` (release
//! mode, statistical); they're slow to run under the regular `cargo
//! test` harness. These tests instead exercise the *public API* the
//! benches depend on, so a regression in those APIs (or in the
//! fixtures used by the benches) is caught by `cargo test`.

use pargen::{ProcessedGrammar, parse_grammar_source, parse_grammar_source_detailed};

/// Mirror of `CALCULATOR_SOURCE` in `benches/parser_codegen.rs`. If you
/// change one, change the other too.
const CALCULATOR_SOURCE: &str = r#"grammar Calculator;

expr: term (('+' | '-') term)*;
term: factor (('*' | '/') factor)*;
factor: NUMBER | '(' expr ')';

NUMBER: [0-9]+;
WS: [ \t\r\n]+ -> skip;
"#;

#[test]
fn bench_fixture_calc_parses_strict() {
    let g = parse_grammar_source(CALCULATOR_SOURCE).expect("calculator parses");
    assert_eq!(g.name, "Calculator");
    let processed = ProcessedGrammar::process(g).expect("process");
    // The bench's full-pipeline path relies on a working analysis pass.
    assert!(processed.implicit_tokens.contains(&"Plus".to_string()));
}

#[test]
fn bench_fixture_calc_parses_recovery() {
    let (_, diags) = parse_grammar_source_detailed(CALCULATOR_SOURCE);
    // For a well-formed grammar the recovery path returns zero diagnostics.
    assert!(diags.is_empty(), "expected 0 diagnostics; got {diags:?}");
}

#[test]
fn bench_full_pipeline_emits_valid_rust() {
    // Same shape as `bench_codegen_rust`'s body — without timing.
    let grammar = parse_grammar_source(CALCULATOR_SOURCE).expect("parse");
    let processed = ProcessedGrammar::process(grammar).expect("analyze");
    let generator = pargen::codegen::get_generator("rust").expect("generator");
    let code = generator.generate(&processed);
    assert!(!code.is_empty());
    assert!(code.contains("pub enum CalculatorTokenKind"));
    assert!(code.contains("pub struct CalculatorLexer"));
    assert!(code.contains("CalculatorVisitor"));
}
