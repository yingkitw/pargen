use pargen::{parse_grammar_source, generate, analysis::ProcessedGrammar, codegen};

fn simple_grammar() -> &'static str {
    r#"grammar Simple;
start: 'hello' ID;
ID: [a-z]+;
WS: [ \t\n]+ -> skip;
"#
}

fn generate_source(lang: &str) -> String {
    let grammar = parse_grammar_source(simple_grammar()).unwrap();
    let processed = ProcessedGrammar::process(grammar).unwrap();
    let generator = codegen::get_generator(lang).unwrap();
    generator.generate(&processed)
}

#[test]
fn test_rust_codegen_contains_structures() {
    let grammar = parse_grammar_source(simple_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    generate(grammar, "rust", temp_dir.path().to_str().unwrap()).unwrap();

    let content = std::fs::read_to_string(temp_dir.path().join("simple_parser.rs")).unwrap();
    assert!(content.contains("pub enum SimpleTokenKind"));
    assert!(content.contains("pub struct SimpleToken"));
    assert!(content.contains("pub struct SimpleLexer"));
    assert!(content.contains("pub struct SimpleParser"));
    assert!(content.contains("pub enum SimpleNode"));
    assert!(content.contains("pub enum SimpleParseError"));
}

#[test]
fn test_go_codegen_contains_structures() {
    let grammar = parse_grammar_source(simple_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    generate(grammar, "go", temp_dir.path().to_str().unwrap()).unwrap();

    let content = std::fs::read_to_string(temp_dir.path().join("simple_parser.go")).unwrap();
    assert!(content.contains("package simple"));
    assert!(content.contains("type SimpleTokenKind"));
    assert!(content.contains("type SimpleLexer"));
    assert!(content.contains("type SimpleParser"));
}

#[test]
fn test_python_codegen_contains_classes() {
    let grammar = parse_grammar_source(simple_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    generate(grammar, "python", temp_dir.path().to_str().unwrap()).unwrap();

    let content = std::fs::read_to_string(temp_dir.path().join("simple_parser.py")).unwrap();
    assert!(content.contains("class SimpleTokenKind"));
    assert!(content.contains("class SimpleToken"));
    assert!(content.contains("class SimpleLexer"));
    assert!(content.contains("class SimpleParser"));
}

#[test]
fn test_typescript_codegen_contains_interfaces() {
    let grammar = parse_grammar_source(simple_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    generate(grammar, "typescript", temp_dir.path().to_str().unwrap()).unwrap();

    let content = std::fs::read_to_string(temp_dir.path().join("simple_parser.ts")).unwrap();
    assert!(content.contains("export enum SimpleTokenKind"));
    assert!(content.contains("export interface SimpleToken"));
    assert!(content.contains("export class SimpleLexer"));
    assert!(content.contains("export class SimpleParser"));
}

#[test]
fn test_java_codegen_contains_classes() {
    let grammar = parse_grammar_source(simple_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    generate(grammar, "java", temp_dir.path().to_str().unwrap()).unwrap();

    let content = std::fs::read_to_string(temp_dir.path().join("simple_parser.java")).unwrap();
    assert!(content.contains("public class SimpleParser"));
    assert!(content.contains("enum SimpleTokenKind"));
}

#[test]
fn test_c_codegen_contains_structures() {
    let grammar = parse_grammar_source(simple_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    generate(grammar, "c", temp_dir.path().to_str().unwrap()).unwrap();

    let h_content = std::fs::read_to_string(temp_dir.path().join("simple_parser.h")).unwrap();
    assert!(h_content.contains("typedef enum"));
    assert!(h_content.contains("typedef struct"));
}

#[test]
fn test_cpp_codegen_contains_classes() {
    let grammar = parse_grammar_source(simple_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    generate(grammar, "cpp", temp_dir.path().to_str().unwrap()).unwrap();

    let content = std::fs::read_to_string(temp_dir.path().join("simple_parser.hpp")).unwrap();
    assert!(content.contains("enum class SimpleTokenKind"));
    assert!(content.contains("struct SimpleToken"));
    assert!(content.contains("class SimpleLexer"));
    assert!(content.contains("class SimpleParser"));
    assert!(content.contains("#include <memory>"));
}

#[test]
fn test_codegen_with_actions() {
    let source = r#"grammar ActionTest;
expr: left=term ('+' right=term { $left + $right })*;
term: NUMBER;
NUMBER: [0-9]+;
WS: [ \t\n]+ -> skip;
"#;
    let grammar = parse_grammar_source(source).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    generate(grammar, "rust", temp_dir.path().to_str().unwrap()).unwrap();

    let content = std::fs::read_to_string(temp_dir.path().join("actiontest_parser.rs")).unwrap();
    assert!(content.contains("ActionTestTokenKind"));
}

#[test]
fn test_codegen_all_languages_for_simple_grammar() {
    let grammar = parse_grammar_source(simple_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().to_str().unwrap();

    for lang in ["rust", "go", "typescript", "python", "java", "c", "cpp", "treesitter"] {
        generate(grammar.clone(), lang, path).expect(&format!("Failed to generate {lang}"));
    }
}

#[test]
fn test_treesitter_codegen_contains_grammar_js() {
    let grammar = parse_grammar_source(simple_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    generate(grammar, "treesitter", temp_dir.path().to_str().unwrap()).unwrap();

    let content = std::fs::read_to_string(temp_dir.path().join("simple_parser.js")).unwrap();
    assert!(content.contains("module.exports = grammar({"));
    assert!(content.contains("name: 'simple',"));
    assert!(content.contains("rules: {"));
    assert!(content.contains("start: $ => seq('hello', $.i_d),"));
    assert!(content.contains("i_d: $ => token(repeat1(/[a-z]+/)),"));
}

#[test]
fn snapshot_simple_rust() {
    insta::assert_snapshot!(generate_source("rust"));
}

#[test]
fn snapshot_simple_go() {
    insta::assert_snapshot!(generate_source("go"));
}

#[test]
fn snapshot_simple_typescript() {
    insta::assert_snapshot!(generate_source("typescript"));
}

#[test]
fn snapshot_simple_python() {
    insta::assert_snapshot!(generate_source("python"));
}

#[test]
fn snapshot_simple_java() {
    insta::assert_snapshot!(generate_source("java"));
}

#[test]
fn snapshot_simple_c() {
    insta::assert_snapshot!(generate_source("c"));
}

#[test]
fn snapshot_simple_cpp() {
    insta::assert_snapshot!(generate_source("cpp"));
}

#[test]
fn snapshot_simple_treesitter() {
    insta::assert_snapshot!(generate_source("treesitter"));
}

fn modes_grammar() -> &'static str {
    r#"grammar StringTest;

start: STR;
WS: [ \t\r\n]+ -> skip;
fragment LETTER: [a-zA-Z];

mode STRINGS;
STR_CHAR: ~'"';
STR_END: '"' -> popMode;
STR_START: '"' -> pushMode(STRINGS);
"#
}

#[test]
fn test_rust_codegen_emits_lexer_mode_enum() {
    let grammar = parse_grammar_source(modes_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    generate(grammar, "rust", temp_dir.path().to_str().unwrap()).unwrap();
    let content = std::fs::read_to_string(temp_dir.path().join("stringtest_parser.rs")).unwrap();
    assert!(content.contains("pub enum StringTestLexerMode"), "expected LexerMode enum, got: {}", content);
    assert!(content.contains("DEFAULT_MODE"), "expected DEFAULT_MODE variant");
    assert!(content.contains("STRINGS"), "expected STRINGS variant");
    assert!(content.contains("mode: StringTestLexerMode"), "expected mode field on Lexer");
    assert!(content.contains("self.mode = StringTestLexerMode::STRINGS"), "expected mode switch on STR_START");
    assert!(content.contains("self.mode = StringTestLexerMode::DEFAULT_MODE"), "expected mode switch on STR_END");
}

#[test]
fn test_rust_codegen_routes_modes_in_dispatch() {
    let grammar = parse_grammar_source(modes_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    generate(grammar, "rust", temp_dir.path().to_str().unwrap()).unwrap();
    let content = std::fs::read_to_string(temp_dir.path().join("stringtest_parser.rs")).unwrap();
    assert!(content.contains("fn try_match_mode_default_mode"), "expected default mode dispatch");
    assert!(content.contains("fn try_match_mode_strings"), "expected strings mode dispatch");
    assert!(content.contains("match self.mode"), "expected mode match in tokenize");
}

#[test]
fn test_other_targets_still_generate_with_modes() {
    let grammar = parse_grammar_source(modes_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().to_str().unwrap();
    for lang in ["go", "typescript", "python", "java", "c", "cpp", "treesitter"] {
        generate(grammar.clone(), lang, path).expect(&format!("{lang} failed with modes grammar"));
    }
}
