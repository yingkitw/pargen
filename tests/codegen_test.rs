use pargen::{parse_grammar_source, generate};

fn simple_grammar() -> &'static str {
    r#"grammar Simple;
start: 'hello' ID;
ID: [a-z]+;
WS: [ \t\n]+ -> skip;
"#
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
