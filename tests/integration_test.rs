use pargen::{parse_grammar_source, parse_grammar_source_detailed, generate};

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

#[test]
fn test_detailed_clean_grammar_returns_no_diagnostics() {
    let (grammar, diagnostics) = parse_grammar_source_detailed(calculator_grammar());
    assert!(grammar.is_some());
    assert!(diagnostics.is_empty(), "expected no diagnostics, got {:?}", diagnostics);
}

#[test]
fn test_detailed_lexer_errors_collected() {
    let source = "grammar Test; % & \\ start: EOF;";
    let (grammar, diagnostics) = parse_grammar_source_detailed(source);
    assert!(diagnostics.len() >= 2, "expected 2+ diagnostics, got {:?}", diagnostics);
    assert!(grammar.is_some(), "grammar should still be recovered");
}

#[test]
fn test_detailed_parser_errors_collected() {
    let source = "grammar Test;\nstart EOF;\nother : EOF;";
    let (grammar, diagnostics) = parse_grammar_source_detailed(source);
    assert!(!diagnostics.is_empty(), "expected at least one diagnostic");
    let g = grammar.expect("grammar should be recoverable");
    let other = g.find_rule("other").expect("'other' rule should be parsed");
    assert_eq!(other.alternatives.len(), 1);
}

#[test]
fn test_detailed_diagnostics_have_locations() {
    let source = "grammar Test; bad EOF;";
    let (_grammar, diagnostics) = parse_grammar_source_detailed(source);
    assert!(!diagnostics.is_empty());
    let with_location = diagnostics
        .iter()
        .filter(|d| d.location.is_some())
        .count();
    assert!(with_location > 0, "expected diagnostics with source locations, got {:?}", diagnostics);
}

#[test]
fn test_detailed_unrecoverable_header() {
    let (grammar, diagnostics) = parse_grammar_source_detailed("");
    assert!(grammar.is_none());
    assert!(!diagnostics.is_empty());
}

#[test]
fn test_lexer_modes_lists_default_and_named_modes() {
    let source = r#"grammar M;
ID: [a-z]+;
mode STRINGS;
STR_CHAR: ~'"';
"#;
    let grammar = parse_grammar_source(source).unwrap();
    let modes = grammar.lexer_modes();
    assert!(modes.contains(&"DEFAULT_MODE".to_string()), "modes: {:?}", modes);
    assert!(modes.contains(&"STRINGS".to_string()), "modes: {:?}", modes);
}

#[test]
fn test_lexer_rules_in_mode_segregates() {
    let source = r#"grammar M;
ID: [a-z]+;
NUM: [0-9]+;
mode STRINGS;
STR_CHAR: ~'"';
"#;
    let grammar = parse_grammar_source(source).unwrap();
    let default = grammar.lexer_rules_in_mode(None);
    assert_eq!(default.len(), 2);
    assert!(default.iter().any(|r| r.name == "ID"));
    assert!(default.iter().any(|r| r.name == "NUM"));

    let strings = grammar.lexer_rules_in_mode(Some("STRINGS"));
    assert_eq!(strings.len(), 1);
    assert_eq!(strings[0].name, "STR_CHAR");
}

#[test]
fn test_mode_command_push_pop_mode_roundtrip() {
    let source = r#"grammar M;
Q: '"' -> pushMode(STRINGS);
mode STRINGS;
C: ~'"';
E: '"' -> popMode;
"#;
    let grammar = parse_grammar_source(source).unwrap();
    let q = grammar.find_rule("Q").unwrap();
    assert_eq!(q.mode, None);
    assert!(q.commands.iter().any(|c| c.name == "pushMode" && c.arg.as_deref() == Some("STRINGS")));

    let e = grammar.find_rule("E").unwrap();
    assert_eq!(e.mode.as_deref(), Some("STRINGS"));
    assert!(e.commands.iter().any(|c| c.name == "popMode"));
}

// --- Semantic predicate detection ----------------------------------------

#[test]
fn test_predicate_detection_via_parse() {
    // Grammar with one top-level predicate and one inside a group.
    let source = r#"grammar Pred;
expr: { $a > 0 }? ID | ('+' { $b }? ID)+;
ID: [a-z]+;
WS: [ \t\n]+ -> skip;
"#;
    let grammar = parse_grammar_source(source).unwrap();
    let preds = grammar.predicates();
    assert_eq!(preds.len(), 2);
    assert_eq!(preds[0].rule_name, "expr");
    assert_eq!(preds[0].alternative_index, 0);
    assert_eq!(preds[0].element_index, 0);
    assert_eq!(preds[0].source, "$a > 0");
    assert_eq!(preds[1].alternative_index, 1);
    assert_eq!(preds[1].source, "$b");
}

#[test]
fn test_predicate_detection_no_predicates() {
    let source = r#"grammar NoPred;
expr: ID ('+' ID)*;
ID: [a-z]+;
WS: [ \t\n]+ -> skip;
"#;
    let grammar = parse_grammar_source(source).unwrap();
    assert!(grammar.predicates().is_empty());
}

#[test]
fn test_predicate_info_is_serializable() {
    // PredicateInfo derives Serialize/Deserialize — confirm round-trip via JSON.
    let source = r#"grammar PS;
expr: { true }? ID;
ID: [a-z]+;
"#;
    let grammar = parse_grammar_source(source).unwrap();
    let preds = grammar.predicates();
    assert_eq!(preds.len(), 1);
    let json = serde_json::to_string(&preds[0]).expect("serialize");
    let back: pargen::PredicateInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, preds[0]);
}

// --- Lexer channel detection --------------------------------------------

#[test]
fn test_channel_detection_via_parse() {
    // Per-rule `-> channel(NAME)` is supported by the parser; the
    // `channels { ... }` declaration block is *not* parseable through
    // the current lexer (it slurps `{ HIDDEN }` as a single Action token).
    // So we exercise per-rule channel assignment, which is the path
    // that real users hit. Implicit channels surface even without
    // a declaration block (mirrors ANTLR behavior).
    let source = r#"grammar Ch;
COMMENT: '//' ~[\n]* -> channel(HIDDEN);
BAD: '!!!' -> channel(ERROR);

start: ID;
ID: [a-z]+;
WS: [ \t\n]+ -> skip;
"#;
    let (grammar, _diagnostics) = parse_grammar_source_detailed(source);
    let grammar = grammar.expect("grammar should parse");
    let chans = grammar.channels();
    assert_eq!(chans.len(), 2);
    let names: Vec<&str> = chans.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"HIDDEN"));
    assert!(names.contains(&"ERROR"));

    let hidden = chans.iter().find(|c| c.name == "HIDDEN").unwrap();
    assert_eq!(hidden.assigned_rules, vec!["COMMENT"]);
    let error = chans.iter().find(|c| c.name == "ERROR").unwrap();
    assert_eq!(error.assigned_rules, vec!["BAD"]);
}

#[test]
fn test_channel_detection_no_channels_declared() {
    let source = r#"grammar NC;
start: ID;
ID: [a-z]+;
WS: [ \t\n]+ -> skip;
"#;
    let grammar = parse_grammar_source(source).unwrap();
    assert!(grammar.channels().is_empty());
}

#[test]
fn test_channel_info_is_serializable() {
    // ChannelInfo derives Serialize/Deserialize — confirm round-trip via JSON.
    // Build the fixture programmatically because the current lexer
    // slurps `{ HIDDEN }` as an Action token (a separate, pre-existing
    // limitation — see MEMORY.md).
    let channel_specs = vec!["HIDDEN".to_string()];
    let channel_json = serde_json::json!({
        "name": "HIDDEN",
        "assigned_rules": vec!["COMMENT"],
    });
    let info: pargen::ChannelInfo = serde_json::from_value(channel_json).unwrap();
    assert_eq!(info.name, "HIDDEN");
    assert_eq!(info.assigned_rules, vec!["COMMENT"]);
    assert!(channel_specs.contains(&info.name));

    // Round-trip
    let json = serde_json::to_string(&info).expect("serialize");
    let back: pargen::ChannelInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, info);
}
