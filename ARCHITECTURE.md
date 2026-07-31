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
│   ├── cpp.rs          — CppGenerator
│   └── treesitter.rs   — TreeSitterGenerator (grammar.js)
├── mcp.rs              — MCP server tools (parse, validate, generate, inspect)
├── bin/pargen-mcp.rs   — MCP binary entry point (stdio transport)
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
│  (grammar)  │  → Vec<G4Token>                       (strict mode)
│             │  G4Lexer::new(source).tokenize_recover()
│             │  → (Vec<G4Token>, Vec<Diagnostic>)   (recovery mode)
└─────────────┘
   │
   ▼
┌─────────────┐
│   Parser    │  G4Parser::new(tokens).parse()
│  (grammar)  │  → Grammar AST                       (strict mode)
│             │  G4Parser::new(tokens).parse_recover()
│             │  → (Option<Grammar>, Vec<Diagnostic>) (recovery mode)
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

### Error Recovery

The strict entry points (`tokenize`, `parse`) return the first error and
stop. The recovery entry points (`tokenize_recover`, `parse_recover`)
collect every diagnostic and continue as far as they can:

- **Lexer** skips an unexpected character and records a diagnostic;
  unterminated strings / actions / comments produce a diagnostic and
  emit an EOF so the parser still has a token stream.
- **Parser** synchronizes to the next `;` after a failed rule, emits a
  partial rule (with empty alternatives) so the outer loop can keep
  going, and records a diagnostic for each failure.

`lib::parse_grammar_source_detailed(source)` ties both passes together and
returns `(Option<Grammar>, Vec<Diagnostic>)`. The grammar is `None` only
when the header itself could not be parsed. The CLI `check` and `generate`
commands use this API; their output lists every diagnostic with severity,
location, and note.

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
    Multiple { diagnostics: ErrorBag },
}
```

- `SourceLocation` tracks `(line, column, file)` and serializes cleanly.
- `Diagnostic` pairs a `DiagnosticSeverity` with a message, an optional `SourceLocation`, and an optional `note` for fix-it hints.
- `ErrorBag` collects multiple `Diagnostic`s. It is the engine behind multi-error reporting — `Error::Multiple` wraps it; `to_error()` collapses a bag of one diagnostic into a single `Error::Parser` for ergonomic single-error use.
- Expected items use `Diagnostic::error(msg).at(line, col).with_note("expected …")` to attach context.

### AST (`grammar::ast`)

| Type | Purpose |
|------|---------|
| `Grammar` | Root container: name, kind, options, rules |
| `Rule` | Single rule with alternatives, modifiers (fragment, skip, channel), mode |
| `Alternative` | One branch of a rule (`\|`-separated) |
| `Element` | One item inside an alternative (rule ref, token ref, literal, charset, etc.) |
| `ElementKind` | Enum: `RuleRef`, `TokenRef`, `Literal`, `CharSet`, `Optional`, `ZeroOrMore`, `Group`, `Dot`, `Range`, `Predicate` |
| `CharSetDef` | Character set definition: list of `CharRange` + negation flag |
| `LexerCommand` | Lexer-side action: `skip`, `channel(NAME)`, `pushMode(NAME)`, `popMode`, `mode(NAME)` |
| `ModeTarget` | Return type of `LexerCommand::mode_target()` — `Named(String)` or `Default` |

All AST types implement `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`.

`Grammar::lexer_modes()` returns every mode referenced (default + named, sorted) and `Grammar::lexer_rules_in_mode(mode)` returns the lexer rules belonging to a given mode. `DEFAULT_MODE` is the implicit mode for rules outside any `mode NAME;` block.

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

## Lexer Modes

Modes are tracked on `Rule::mode` (an `Option<String>`). The parser keeps a
`current_mode` cursor that updates when it sees `mode NAME;`; each
subsequent rule inherits that mode. `Grammar::lexer_modes()` is the
canonical list of modes (default + any named modes referenced by rules
or by `pushMode`/`mode` commands).

The Rust target emits a `LexerMode` enum, a `mode` field on the lexer,
and a `match self.mode` dispatch in `tokenize` that routes to a per-mode
`try_match_mode_<name>` function. After a rule matches, the dispatch
applies the rule's lexer commands — `pushMode(NAME)` and `mode(NAME)`
set `self.mode = LexerMode::NAME`; `popMode` resets to `DEFAULT_MODE`.

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
pub use core::{Error, Result, Diagnostic, DiagnosticSeverity, SourceLocation, ErrorBag};
pub use core::{GrammarParser, SemanticAnalyzer, CodeGenerator};
pub use grammar::{Grammar, GrammarKind, Rule, Alternative, Element, ElementKind};
pub use analysis::ProcessedGrammar;

pub fn parse_grammar_file(path: &str) -> anyhow::Result<Grammar>;
pub fn parse_grammar_source(source: &str) -> anyhow::Result<Grammar>;
pub fn parse_grammar_source_detailed(source: &str) -> (Option<Grammar>, Vec<Diagnostic>);
pub fn generate(grammar: Grammar, lang: &str, output_dir: &str) -> anyhow::Result<()>;
```

These functions are the primary programmatic interface. `parse_grammar_source` is the strict entry point; `parse_grammar_source_detailed` is the recovery entry point — it returns the partial grammar plus every diagnostic, never `Err`.

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
| `src/grammar/lexer.rs` (`#[cfg(test)]`) | 31 | Token kinds, literals, comments, errors, proptests, recovery |
| `src/grammar/parser.rs` (`#[cfg(test)]`) | 39 | Grammar headers, rules, alternatives, labels, actions, fragments, error cases, recovery, modes |
| `src/analysis/left_rec.rs` (`#[cfg(test)]`) | 7 | Direct/indirect left recursion, empty alternatives |
| `src/core/error.rs` (`#[cfg(test)]`) | 20 | Display, IO conversion, serde round-trip, clone/equality, ErrorBag + Diagnostic builders |
| `tests/integration_test.rs` | 30 | End-to-end parse + generate for all 8 targets, error cases, `parse_grammar_source_detailed`, mode helpers |
| `tests/codegen_test.rs` | 21 | Structure verification + insta snapshots per language, mode-aware Rust codegen |

**148 tests total**, all run with `cargo test`.

---

## Dependencies

| Crate | Role |
|-------|------|
| `clap` | CLI argument parsing |
| `anyhow` | Ergonomic error propagation in CLI / high-level API |
| `thiserror` | Structured `Error` enum with `Display` |
| `serde` + `serde_json` | AST serialization / diagnostics |
| `tracing` + `tracing-subscriber` | Structured logging |
| `insta` (dev) | Snapshot testing (ready for codegen snapshots) |
| `criterion` (dev) | Benchmark harness (ready for parser/codegen benchmarks) |
| `tempfile` (dev) | Integration test output directories |
| `proptest` (dev) | Property-based lexer robustness tests |
| `rmcp` + `tokio` + `schemars` | MCP server (`pargen-mcp` binary) |

---

## Known Limitations & Extension Points

1. **Lexer / Parser**: hand-written recursive-descent. Future work could adopt a generated or table-driven approach.
2. **Options block**: the lexer tokenizes `{ ... }` contents as action blocks; ANTLR4-style `options { ... }` parsing is partially supported.
3. **Lexer modes & channels**: lexer `mode` declarations are parsed and the Rust target emits mode-aware dispatch. The other 7 targets still treat all rules as default-mode. Channel support (e.g. `-> channel(HIDDEN)`) is still read into the AST but not emitted.
4. **Semantic predicates**: `{ ... }?` and `{ ... }^` predicates are parsed into the AST but not enforced by generated parsers.
5. **`parse` CLI command**: placeholder — requires a generated parser to actually parse input text.

---

## MCP Server (`pargen-mcp`)

The `src/mcp.rs` module exposes five tools over stdio transport:

| Tool | Purpose |
|------|---------|
| `parse_grammar` | Parse grammar text, return AST summary |
| `validate_grammar` | Parse + analyze, return diagnostics |
| `generate_code` | Full pipeline to any target language |
| `get_grammar_info` | Detailed rule lists and options |
| `list_target_languages` | List all 8 supported targets |

---

Last updated: 2026-07-31
