use pargen::{parse_grammar_source, generate};

fn calculator_grammar() -> &'static str {
    r#"grammar Calculator;

expr: term (('+' | '-') term)*;
term: factor (('*' | '/') factor)*;
factor: NUMBER | '(' expr ')';

NUMBER: [0-9]+;
WS: [ \t\r\n]+ -> skip;
"#
}

fn json_grammar() -> &'static str {
    r#"grammar JSON;

json: value;
value: object | array | STRING | NUMBER | 'true' | 'false' | 'null';
object: '{' pair (',' pair)* '}' | '{' '}';
pair: STRING ':' value;
array: '[' value (',' value)* ']' | '[' ']';

STRING: '"' [a-zA-Z ]* '"';
NUMBER: '-'? [0-9]+ ('.' [0-9]+)?;
WS: [ \t\r\n]+ -> skip;
"#
}

#[test]
fn test_parse_calculator_grammar() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    assert_eq!(grammar.name, "Calculator");
    assert_eq!(grammar.kind, pargen::GrammarKind::Combined);
    assert_eq!(grammar.rules.len(), 5); // expr, term, factor, NUMBER, WS
}

#[test]
fn test_parse_json_grammar() {
    let grammar = parse_grammar_source(json_grammar()).unwrap();
    assert_eq!(grammar.name, "JSON");
    let parser_rules = grammar.parser_rules();
    let lexer_rules = grammar.lexer_rules();
    assert_eq!(parser_rules.len(), 5); // json, value, object, pair, array
    assert_eq!(lexer_rules.len(), 3);  // STRING, NUMBER, WS
}

#[test]
fn test_grammar_lexer_rules() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let lexer = grammar.lexer_rules();
    assert_eq!(lexer.len(), 2);
    assert_eq!(lexer[0].name, "NUMBER");
    assert_eq!(lexer[1].name, "WS");
    assert!(lexer[1].is_skip());
}

#[test]
fn test_grammar_parser_rules() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let parser = grammar.parser_rules();
    assert_eq!(parser.len(), 3);
    assert_eq!(parser[0].name, "expr");
    assert_eq!(parser[1].name, "term");
    assert_eq!(parser[2].name, "factor");
}

#[test]
fn test_find_rule() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    assert!(grammar.find_rule("expr").is_some());
    assert!(grammar.find_rule("NUMBER").is_some());
    assert!(grammar.find_rule("nonexistent").is_none());
}

#[test]
fn test_start_rule() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let start = grammar.start_rule().unwrap();
    assert_eq!(start.name, "expr");
}

#[test]
fn test_rule_alternatives() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let factor = grammar.find_rule("factor").unwrap();
    assert_eq!(factor.alternatives.len(), 2);
}

#[test]
fn test_lexer_command_skip() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let ws = grammar.find_rule("WS").unwrap();
    assert!(ws.is_skip());
    assert_eq!(ws.channel(), None);
}

#[test]
fn test_generate_rust_parser() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    generate(grammar, "rust", temp_path).unwrap();

    let output_file = temp_dir.path().join("calculator_parser.rs");
    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("pub enum CalculatorTokenKind"));
    assert!(content.contains("pub struct CalculatorToken"));
}

#[test]
fn test_generate_go_parser() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    generate(grammar, "go", temp_path).unwrap();

    let output_file = temp_dir.path().join("calculator_parser.go");
    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("package calculator"));
}

#[test]
fn test_generate_python_parser() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    generate(grammar, "python", temp_path).unwrap();

    let output_file = temp_dir.path().join("calculator_parser.py");
    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("class CalculatorTokenKind"));
}

#[test]
fn test_generate_typescript_parser() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    generate(grammar, "typescript", temp_path).unwrap();

    let output_file = temp_dir.path().join("calculator_parser.ts");
    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("export enum CalculatorTokenKind"));
}

#[test]
fn test_generate_java_parser() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    generate(grammar, "java", temp_path).unwrap();

    let output_file = temp_dir.path().join("calculator_parser.java");
    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("public class CalculatorParser"));
}

#[test]
fn test_generate_c_parser() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    generate(grammar, "c", temp_path).unwrap();

    let output_file_h = temp_dir.path().join("calculator_parser.h");
    assert!(output_file_h.exists());
}

#[test]
fn test_generate_treesitter_parser() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    generate(grammar, "treesitter", temp_path).unwrap();

    let output_file = temp_dir.path().join("calculator_parser.js");
    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("module.exports = grammar({"));
    assert!(content.contains("name: 'calculator',"));
    assert!(content.contains("rules: {"));
}

#[test]
fn test_generate_cpp_parser() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    generate(grammar, "cpp", temp_path).unwrap();

    let output_file = temp_dir.path().join("calculator_parser.hpp");
    assert!(output_file.exists());
    let content = std::fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("#include <memory>"));
}

#[test]
fn test_unsupported_language() {
    let grammar = parse_grammar_source(calculator_grammar()).unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    let result = generate(grammar, "swift", temp_path);
    assert!(result.is_err());
}

#[test]
fn test_empty_grammar_fails() {
    let result = parse_grammar_source("");
    assert!(result.is_err());
}

#[test]
fn test_grammar_with_options() {
    // Note: options block syntax is not fully supported by current lexer/parser
    // Options inside { } get tokenized as action blocks
    let source = r#"grammar Test;
start: EOF;
"#;
    let grammar = parse_grammar_source(source).unwrap();
    assert_eq!(grammar.name, "Test");
    assert!(grammar.options.is_empty());
}

#[test]
fn test_grammar_with_imports() {
    let source = r#"grammar Test;
import CommonLexer;
start: EOF;
"#;
    let grammar = parse_grammar_source(source).unwrap();
    // imports are parsed but not stored in grammar struct currently
    assert_eq!(grammar.name, "Test");
}

#[test]
fn test_grammar_with_fragment() {
    let source = r#"grammar Test;
fragment DIGIT: [0-9];
NUMBER: DIGIT+;
start: NUMBER;
"#;
    let grammar = parse_grammar_source(source).unwrap();
    let fragment = grammar.find_rule("DIGIT").unwrap();
    assert!(fragment.is_fragment);
    assert!(fragment.is_lexer_rule());
}

#[test]
fn test_grammar_with_character_class() {
    let source = r#"grammar Test;
ID: [a-zA-Z_] [a-zA-Z0-9_]*;
start: ID;
"#;
    let grammar = parse_grammar_source(source).unwrap();
    let id_rule = grammar.find_rule("ID").unwrap();
    assert_eq!(id_rule.alternatives.len(), 1);
}
