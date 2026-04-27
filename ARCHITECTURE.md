# pargen Architecture

## Overview

`pargen` is a parser generator that reads ANTLR4 grammar files (`.g4`) and emits parsers in multiple target languages. It is designed around a clear pipeline: **lex → parse → analyze → generate**.

Key design principles:

- **Modular by domain**: each pipeline phase lives in its own module (`grammar`, `analysis`, `codegen`).
- **Trait-based extension**: adding a new target language requires implementing a single trait.
- **Serde-ready AST**: all AST types derive `Serialize` and `Deserialize` for programmatic inspection and future tooling.
- **Structured errors**: `thiserror`-based error enum with source locations and diagnostics.
- **Observability**: `tracing` is used throughout instead of `println!`.

---

## Module Layout

```
src/
├── core/
│   ├── error.rs    — Error, Result, SourceLocation, Diagnostic, DiagnosticSeverity
│   ├── traits.rs   — GrammarParser, SemanticAnalyzer, CodeGenerator traits
│   └── mod.rs      — re-exports
├── grammar/
│   ├── ast.rs      — Grammar, Rule, Alternative, Element, ElementKind, CharSetDef, etc.
│   ├── lexer.rs    — G4Lexer: tokenizes ANTLR4 source into G4TokenKind stream
│   ├── parser.rs   — G4Parser: recursive-descent parser producing Grammar AST
│   └── mod.rs      — re-exports
├── analysis/
│   ├── first_follow.rs — FIRST/FOLLOW sets, implicit-token collection, ProcessedGrammar
│   ├── left_rec.rs     — left-recursion detection & elimination
│   └── mod.rs          — re-exports
├── codegen/
│   ├── mod.rs          — CodeGenerator trait, CodeWriter, get_generator, generate_output
│   ├── rust.rs         — RustGenerator
│   ├── go.rs           — GoGenerator
│   ├── typescript.rs   — TypeScriptGenerator
│   ├── python.rs       — PythonGenerator
│   ├── java.rs         — JavaGenerator
│   ├── c.rs            — CGenerator
│   └── cpp.rs          — CppGenerator
├── lib.rs              — public API: parse_grammar_file, parse_grammar_source, generate
└── main.rs             — CLI entry point (clap subcommands: generate, init, parse, check)
```

---

## Data Flow

```
.g4 file
   │
   ▼
┌─────────────┐
│   Lexer     │  G4Lexer::new(source).tokenize()
│  (grammar)  │  → Vec<G4Token>
└─────────────┘
   │
   ▼
┌─────────────┐
│   Parser    │  G4Parser::new(tokens, source).parse()
│  (grammar)  │  → Grammar AST
└─────────────┘
   │
   ▼
┌─────────────┐
│  Analysis   │  ProcessedGrammar::process(grammar)
│  (analysis) │  → FIRST/FOLLOW, left-rec elimination, implicit tokens
└─────────────┘
   │
   ▼
┌─────────────┐
│   CodeGen   │  get_generator(lang).generate(&processed)
│  (codegen)  │  → source code string
└─────────────┘
   │
   ▼
 output file
```

---

## Core Types

### Error (`core::error`)

```rust
pub enum Error {
    Io(String),
    Lexer  { location: SourceLocation, message: String },
    Parser { location: SourceLocation, message: String },
    Analysis(String),
    Codegen(String),
    UnsupportedLanguage(String),
    ValidationFailed { count: usize },
}
```

- `SourceLocation` tracks `(line, column, file)` and serializes cleanly.
- `Diagnostic` pairs a `DiagnosticSeverity` with a message and optional location.

### AST (`grammar::ast`)

| Type | Purpose |
|------|---------|
| `Grammar` | Root container: name, kind, options, rules |
| `Rule` | Single rule with alternatives, modifiers (fragment, skip, channel) |
| `Alternative` | One branch of a rule (`\|`-separated) |
| `Element` | One item inside an alternative (rule ref, token ref, literal, charset, etc.) |
| `ElementKind` | Enum: `RuleRef`, `TokenRef`, `Literal`, `CharSet`, `Optional`, `ZeroOrMore`, `OneOrMore`, `Group`, `Dot`, `Range`, `Predicate` |
| `CharSetDef` | Character set definition: list of `CharRange` + negation flag |

All AST types implement `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`.

### ProcessedGrammar (`analysis::first_follow`)

```rust
pub struct ProcessedGrammar {
    pub grammar: Grammar,
    pub first_sets: HashMap<String, HashSet<String>>,
    pub follow_sets: HashMap<String, HashSet<String>>,
    pub transformed_rules: Vec<Rule>,   // left-recursion eliminated
    pub implicit_tokens: Vec<String>,   // string literals promoted to tokens
}
```

---

## Lexer (`grammar::lexer`)

`G4Lexer` is a hand-written character-at-a-time lexer. It recognizes:

- Keywords (`grammar`, `lexer`, `parser`, `fragment`, `mode`, `skip`, `channel`, etc.)
- Identifiers and string literals (single- and double-quoted)
- Character sets `[a-zA-Z]`
- Action blocks `{ ... }`
- Operators and punctuation (`;`, `:`, `|`, `*`, `+`, `?`, `->`, etc.)
- Line (`//`) and block (`/* */`) comments

Tokens carry `line`, `column`, and `offset` for error reporting.

---

## Parser (`grammar::parser`)

`G4Parser` is a recursive-descent parser over the token stream. It builds:

1. Grammar header (`grammar Name;`)
2. Options block
3. Token / channel declarations
4. Imports
5. Actions (`@name { ... }`, `@scope::name { ... }`)
6. Rules (lexer rules with uppercase names, parser rules with lowercase names)

Each rule contains one or more `Alternative`s, each containing `Element`s with `ElementKind`s.

---

## Analysis (`analysis`)

### FIRST / FOLLOW Sets

Computed per-rule over the transformed grammar. Used by codegen to determine parse decisions (look-ahead).

### Left-Recursion Elimination

`left_rec.rs` provides:

- `has_direct_left_recursion(rule)` — checks if first element references the rule itself.
- `transform_direct_left_recursion(rule)` — converts `A → A α \| β` into `A → β A'` and `A' → α A' \| ε`.
- `eliminate_indirect_lr(rules)` — resolves mutual/indirect left recursion via rule reordering and substitution.

### Implicit Tokens

String literals used directly in parser rules (e.g., `'+'`, `'if'`) are collected and promoted to implicit token names (`T_1`, `T_2`, etc.) so the generated lexer has something to match.

---

## Code Generation (`codegen`)

### `CodeGenerator` Trait

```rust
pub trait CodeGenerator {
    fn lang_name(&self) -> &str;
    fn file_extension(&self) -> &str;
    fn generate(&self, grammar: &ProcessedGrammar) -> String;
}
```

Each target language lives in its own file (`rust.rs`, `go.rs`, etc.) and implements this trait.

### `CodeWriter`

A small indentation helper used by every generator:

```rust
let mut w = CodeWriter::new("    ");
w.line("fn foo() {");
w.indent();
w.line("bar();");
w.dedent();
w.line("}");
let code = w.to_string();
```

### Utility Functions

| Function | Purpose |
|----------|---------|
| `snake_case` | `RuleName` → `rule_name` |
| `pascal_case` | `rule_name` → `RuleName` |
| `camel_case` | `rule_name` → `ruleName` |
| `escape_string` | `"` → `\"`, `\n` → `\n`, etc. |
| `collect_tokens` | Gather lexer rules + implicit tokens |

### Generating for a New Language

1. Create `src/codegen/<lang>.rs`.
2. Define a struct (e.g., `LangGenerator`) and implement `CodeGenerator`.
3. Register it in `codegen::get_generator`:
   ```rust
   "lang" => Ok(Box::new(lang::LangGenerator)),
   ```
4. Add a snapshot or integration test in `tests/`.

---

## Public API (`lib.rs`)

```rust
pub use core::{Error, Result, Diagnostic, DiagnosticSeverity};
pub use core::{GrammarParser, SemanticAnalyzer, CodeGenerator};
pub use grammar::{Grammar, GrammarKind, Rule, Alternative, Element, ElementKind};
pub use analysis::ProcessedGrammar;

pub fn parse_grammar_file(path: &str) -> anyhow::Result<Grammar>;
pub fn parse_grammar_source(source: &str) -> anyhow::Result<Grammar>;
pub fn generate(grammar: Grammar, lang: &str, output_dir: &str) -> anyhow::Result<()>;
```

These three functions are the primary programmatic interface. `parse_grammar_source` is especially useful for tests.

---

## CLI (`main.rs`)

Subcommands:

| Command | Purpose |
|---------|---------|
| `generate` | Parse `.g4` and emit parser in target language |
| `init` | Scaffold a minimal `.g4` file |
| `parse` | Parse grammar + (placeholder) parse input with generated parser |
| `check` | Parse grammar and report rule counts / diagnostics |

`tracing_subscriber::fmt::init()` enables `RUST_LOG=info pargen ...` for structured logging.

---

## Testing

Tests are organized by scope:

| Test File | Count | Scope |
|-----------|-------|-------|
| `src/grammar/lexer.rs` (`#[cfg(test)]`) | 24 | Token kinds, literals, comments, errors |
| `src/grammar/parser.rs` (`#[cfg(test)]`) | 27 | Grammar headers, rules, alternatives, labels, actions, fragments, error cases |
| `src/analysis/left_rec.rs` (`#[cfg(test)]`) | 8 | Direct/indirect left recursion, empty alternatives |
| `src/core/error.rs` (`#[cfg(test)]`) | 13 | Display, IO conversion, serde round-trip, clone/equality |
| `tests/integration_test.rs` | 21 | End-to-end parse + generate for all 7 languages, error cases |
| `tests/codegen_test.rs` | 9 | Structure verification of generated code per language |

**102 tests total**, all run with `cargo test`.

---

## Dependencies

| Crate | Role |
|-------|------|
| `clap` | CLI argument parsing |
| `anyhow` | Ergonomic error propagation in CLI / high-level API |
| `thiserror` | Structured `Error` enum with `Display` |
| `serde` + `serde_json` | AST serialization / diagnostics |
| `tracing` + `tracing-subscriber` | Structured logging |
| `insta` (dev) | Snapshot testing (ready for future codegen snapshots) |
| `criterion` (dev) | Benchmark harness (ready for future benchmarks) |
| `tempfile` (dev) | Integration test output directories |

---

## Known Limitations & Extension Points

1. **Lexer / Parser**: hand-written recursive-descent. Future work could adopt a generated or table-driven approach.
2. **Error recovery**: parser stops on first error. Multi-error reporting is a future enhancement.
3. **Options block**: the lexer tokenizes `{ ... }` contents as action blocks; ANTLR4-style `options { ... }` parsing is not yet fully supported.
4. **MCP server**: a `pargen-mcp` binary exposing `parse_grammar`, `validate_grammar`, and `generate_code` tools is planned.
5. **Property-based testing**: `proptest` integration for lexer / parser robustness is planned.
6. **Tree-sitter target**: emitting `grammar.js` from ANTLR4 input is a potential future target.

---

Last updated: 2026-04-27
