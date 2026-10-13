//! Criterion benchmarks for the pargen pipeline.
//!
//! Four benches cover the dominant phases users hit:
//!   1. Strict parse of a small grammar (calculator).
//!   2. Recovery parse of the same grammar (multi-error path).
//!   3. Full codegen pipeline (parse → analyze → generate) for one
//!      target language, with output discarded — measures the
//!      generator, not the file system.
//!   4. Scalability probe: synthesize a larger grammar with N
//!      implicit-token rules and parse it.
//!
//! Run with `cargo bench --bench parser_codegen`. Build-only check:
//! `cargo bench --bench parser_codegen -- --quick` or invoke the smoke
//! test in `tests/codegen_test.rs`-style via `cargo test --lib`.
//!
//! The benches use `criterion::black_box` to keep the optimizer honest
//! about inputs/outputs.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pargen::{ProcessedGrammar, parse_grammar_source, parse_grammar_source_detailed};

/// A small but realistic grammar — same source as the integration tests'
/// `calculator_grammar()` so the benchmark measures something the
/// project actually exercises.
const CALCULATOR_SOURCE: &str = r#"grammar Calculator;

expr: term (('+' | '-') term)*;
term: factor (('*' | '/') factor)*;
factor: NUMBER | '(' expr ')';

NUMBER: [0-9]+;
WS: [ \t\r\n]+ -> skip;
"#;

/// Run the strict (non-recovery) parser on the calculator grammar.
fn bench_parse_strict(c: &mut Criterion) {
    let source = CALCULATOR_SOURCE;
    c.bench_function("parse_grammar_source_strict", |b| {
        b.iter(|| parse_grammar_source(black_box(source)).expect("strict parse"));
    });
}

/// Recovery-mode parse of the calculator grammar. This exercises the
/// `parse_grammar_source_detailed` path including `tokenize_recover`
/// and `parse_recover`.
fn bench_parse_recovery(c: &mut Criterion) {
    let source = CALCULATOR_SOURCE;
    c.bench_function("parse_grammar_source_detailed", |b| {
        b.iter(|| {
            let (g, diags) = parse_grammar_source_detailed(black_box(source));
            // Ensure the compiler doesn't drop the work.
            black_box(g.is_some());
            black_box(diags.len());
        });
    });
}

/// Full pipeline: strict parse → analysis → codegen into Rust. Output
/// is written to a `tempfile::TempDir` because the codegen API requires
/// a path; we drop the dir at the end of each iteration so benchmarks
/// don't accumulate state on disk.
fn bench_codegen_rust(c: &mut Criterion) {
    let source = CALCULATOR_SOURCE;
    c.bench_function("codegen_rust_full_pipeline", |b| {
        b.iter(|| {
            // Re-parse every iteration to keep the pipeline shape
            // identical to a real user invocation. (Codegen itself
            // takes a `Grammar`, so re-parsing measures the same
            // volume of work the user would do.)
            let grammar = parse_grammar_source(source).expect("parse");
            let processed = ProcessedGrammar::process(grammar).expect("analyze");
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().to_str().expect("path");
            let generator = pargen::codegen::get_generator("rust").expect("generator");
            let code = generator.generate(&processed);
            black_box(code.len());
            // dir dropped here — file cleaned up by TempDir.
            let _ = path; // suppress unused-warning while keeping the path semantics
        });
    });
}

/// Scalability probe: synthesize a grammar with N implicit tokens
/// (arithmetic operators + a nested rule) and parse it. Captures how
/// the parser scales with token count, which matters for grammars
/// pulled from real-world projects.
fn bench_scaling_grammar_parse(c: &mut Criterion) {
    let n = 64usize;
    let source = synthesize_grammar(n);

    c.bench_function(&format!("parse_grammar_source_scaling_n{n}"), |b| {
        b.iter(|| parse_grammar_source(black_box(source.as_str())).expect("strict parse"));
    });
}

/// Build a synthetic grammar with `n` implicit tokens: an `expr` rule
/// that accepts any of the N operators in a sequence, plus a `NUMBER`
/// terminal. Used only by the scaling benchmark.
fn synthesize_grammar(n: usize) -> String {
    let mut s = String::from("grammar Synth;\n\nexpr: NUMBER (");
    let ops: Vec<String> = (0..n)
        .map(|i| format!("'+{}' ", i))
        .collect();
    s.push_str(&ops.join("| "));
    s.push_str(") NUMBER;\n\nNUMBER: [0-9]+;\nWS: [ \\t\\r\\n]+ -> skip;\n");
    s
}

criterion_group!(
    name = parser_codegen_benches;
    config = Criterion::default().sample_size(50);
    targets =
        bench_parse_strict,
        bench_parse_recovery,
        bench_codegen_rust,
        bench_scaling_grammar_parse,
);

criterion_main!(parser_codegen_benches);
