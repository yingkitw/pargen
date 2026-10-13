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
| `PredicateInfo` | One `{ ... }?` predicate location: `rule_name`, `alternative_index`, `element_index`, `source`. Returned by `Grammar::predicates()`. |
| `ChannelInfo` | One lexer channel: `name` (declared via `channels { HIDDEN }` or referenced implicitly by `-> channel(NAME)`) plus `assigned_rules: Vec<String>`. Returned by `Grammar::channels()`. |

All AST types implement `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`.

`Grammar::lexer_modes()` returns every mode referenced (default + named, sorted) and `Grammar::lexer_rules_in_mode(mode)` returns the lexer rules belonging to a given mode. `DEFAULT_MODE` is the implicit mode for rules outside any `mode NAME;` block. `Grammar::predicates()` walks every alternative (including inside `( ... )`, `( ... )?`, `( ... )*`, `( ... )+`) and reports each `{ ... }?` predicate as a `PredicateInfo`. Predicates are detected for reporting and (eventually) enforcement; today the generated parsers skip them at runtime — see Known Limitations. `Grammar::channels()` walks `channel_specs` plus every lexer rule's `-> channel(NAME)` command and reports each channel as a `ChannelInfo`. Declared channels come first in declaration order; implicitly-referenced channels are appended in the order first seen, mirroring ANTLR's implicit-channel behavior. Channel routing is reporting-only today — see Known Limitations.

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
| `charset_match_expr` (Rust target only) | Build an inline Rust boolean expression for matching a `char` against a `CharSetDef`. Emits `'a'..='z'` for multi-char ranges and `c == '\t'` for single-char, OR-combined and negated as needed. The generated lexer uses this directly — no helper method or global constant is required. |

### Generating for a New Language

1. Create `src/codegen/<lang>.rs`.
2. Define a struct (e.g., `LangGenerator`) and implement `CodeGenerator`.
3. Register it in `codegen::get_generator`:
   ```rust
   "lang" => Ok(Box::new(lang::LangGenerator)),
   ```
4. Add a snapshot or integration test in `tests/`.

---

## Visitors and Listeners (Rust target)

The Rust target emits two tree-walker traits and two traversal functions on top of the existing `<Name>Node` CST. They are generated by `gen_visitor`, `gen_listener`, and `gen_walker` in `src/codegen/rust.rs`.

**`<Name>Visitor`** — one default-no-op method per parser rule plus `visit_token`:
```rust
pub trait SimpleVisitor {
    fn visit_start(&mut self, _node: &SimpleNode) {}
    fn visit_token(&mut self, _node: &SimpleNode) {}
}
```

**`<Name>Listener`** — `enter_<rule>` / `exit_<rule>` per parser rule plus `visit_token`. Default impls are no-ops:
```rust
pub trait SimpleListener {
    fn enter_start(&mut self, _node: &SimpleNode) {}
    fn exit_start(&mut self, _node: &SimpleNode) {}
    fn visit_token(&mut self, _node: &SimpleNode) {}
}
```

**Walkers** — both operate on `&<Name>Node`, dispatch on the variant, and recurse into `children`:

- `walk_<snake_grammar_name>(visitor: &mut V, &<Name>Node)` calls `visit_<rule>` *before* recursing.
- `walk_<snake_grammar_name>_listen(listener: &mut L, &<Name>Node)` brackets children between `enter_<rule>` and `exit_<rule>`.

Walkers take `&mut dyn` via `?Sized` so they accept trait objects as well as owned impls.

**Scope**: visitor/listener/walker code is emitted *only* by the Rust target. The other 7 targets emit language-idiomatic ASTs that have no equivalent typed enum; deferring their walkers until at least one other target grows a typed CST enum. Both traits are zero-cost to implement (override only what you need) and the walker functions are total — exhaustive match on every `<Name>Node` variant is guaranteed at generate time.

Example usage on the calculator grammar:
```rust
struct PrintDots;
impl CalculatorVisitor for PrintDots {
    fn visit_expr(&mut self, n: &CalculatorNode) {
        println!("[enter expr ({} children)]", match n {
            CalculatorNode::expr { children } => children.len(),
            _ => 0,
        });
    }
}
let tree = parser.parse_expr()?;
walk_calculator(&mut PrintDots, &tree);
```

---

## Public API (`lib.rs`)

```rust
pub use core::{Error, Result, Diagnostic, DiagnosticSeverity, SourceLocation, ErrorBag};
pub use core::{GrammarParser, SemanticAnalyzer, CodeGenerator};
pub use grammar::{Grammar, GrammarKind, Rule, Alternative, Element, ElementKind, PredicateInfo, ChannelInfo};
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
| `tests/integration_test.rs` | 33 | End-to-end parse + generate for all 8 targets, error cases, `parse_grammar_source_detailed`, mode helpers, semantic predicate detection |
| `tests/codegen_test.rs` | 29 | Structure verification + insta snapshots per language, mode-aware Rust codegen, visitor/listener/walker codegen (Rust target only), charset lexer regression tests |
| `tests/bench_smoke.rs` | 3 | Smoke-tests the public API the criterion benches depend on (`parse_grammar_source`, `parse_grammar_source_detailed`, `ProcessedGrammar::process`, `codegen::get_generator`). Ensures a regression in those APIs surfaces in `cargo test`, not only `cargo bench`. |

**175 tests total**, all run with `cargo test`.

---

## Benchmarks (`benches/`)

`benches/parser_codegen.rs` is a single criterion bench target with four phases of the pipeline measured independently:

| Target | Phase Measured |
|---|---|
| `parse_grammar_source_strict` | Lex + parse a small grammar with bail-on-first-error |
| `parse_grammar_source_detailed` | Same input through the recovery / multi-error path |
| `codegen_rust_full_pipeline` | Parse → analysis → Rust code generation in one shot |
| `parse_grammar_source_scaling_n64` | Synthetic grammar with 64 implicit tokens to probe scaling |

Run with `cargo bench --bench parser_codegen`. The bench uses `criterion::black_box` to prevent the optimizer from shortcutting inputs/outputs, and `sample_size(50)` to keep CI-bound runs bounded when reducing warm-up time.

Initial baseline numbers (release profile, `--warm-up-time 0.1 --measurement-time 0.2`, 2026-10-13):

| Phase | Time |
|---|---|
| strict parse | ~4.2 µs |
| recovery parse | ~4.1 µs |
| full Rust pipeline | ~260 µs |
| scaling N=64 | ~11.3 µs |

Codegen is ~58× slower than parsing — the dominant cost is `String` allocation per line via `CodeWriter`. Future work to widen this benchmark: add all 8 codegen targets, add a hand-written-parser comparison, add memory-footprint benches, and add a regression threshold so CI catches sub-1.5× slowdowns.

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
| `criterion` (dev) | Benchmark harness. Wired via `benches/parser_codegen.rs` (4 targets). Smoke-tested via `tests/bench_smoke.rs` integration tests. Run with `cargo bench --bench parser_codegen`. Quick-mode initial baseline (~release profile): strict parse 4.2 µs, recovery parse 4.1 µs, full codegen pipeline 260 µs, scaling N=64 11.3 µs. |
| `tempfile` (dev) | Integration test output directories |
| `proptest` (dev) | Property-based lexer robustness tests |
| `rmcp` + `tokio` + `schemars` | MCP server (`pargen-mcp` binary) |

---

## Known Limitations & Extension Points

1. **Lexer / Parser**: hand-written recursive-descent. Future work could adopt a generated or table-driven approach.
2. **Options block**: the lexer tokenizes `{ ... }` contents as action blocks; ANTLR4-style `options { ... }` parsing is partially supported.
3. **Lexer modes & channels**: lexer `mode` declarations are parsed and the Rust target emits mode-aware dispatch. The other 7 targets still treat all rules as default-mode. Channel support (`-> channel(HIDDEN)`) is detected and surfaced via `Grammar::channels()` (CLI + MCP), but generated parsers do not route tokens by channel — full routing requires touching every generated `parse_<rule>` plus the lexer's token-emit loop.
4. **Semantic predicates**: `{ ... }?` predicates are detected and exposed by `Grammar::predicates()` (with CLI/MCP surfacing), but generated parsers do not execute them to disambiguate alternatives. Full enforcement requires threading a predicate solver through every `parse_<rule>` call — deferred to a future iteration.
5. **`parse` CLI command**: placeholder — requires a generated parser to actually parse input text.
6. **Generated Rust lexer position-restore (investigated, benign)**: `src/codegen/rust.rs::gen_lexer_match_fn` emits four `return None;` paths that don't explicitly restore `self.pos = saved_pos`. Investigation (2026-10-13) showed this is benign today: `peek_at` doesn't advance `self.pos`, and the `len == 0` branch only fires at EOF where `self.pos == input.len()` already. `gen_lexer_element_match` (a sibling path used inside OneOrMore blocks) does emit the restore because it can run in a context where partially-advanced state would matter, but that path was also unused-by-current-tests. No fix applied — per Mature Module Policy, drive-by changes to mature modules need a concrete reproducer. Re-evaluate if a future codegen path starts exercising peek_at-with-side-effects or a non-EOF `len == 0` branch.
7. **`channels { HIDDEN }` declaration block not parseable via strict lexer**: the lexer's `read_action_block` slurps any `{ ... }` content into a single `Action` token — including the `channels { HIDDEN, ERROR }` declaration block. The grammar-parser paths for `parse_channels` and `parse_channels_recover` exist and are reachable only via the *recovery* lexer path (`parse_grammar_source_detailed`). Per-rule `-> channel(NAME)` is supported by the strict lexer because it doesn't open a brace. To properly parse `channels { ... }`, the lexer would need a mode/context that distinguishes option/channel/mode-block braces from action braces — out of scope for current iterations.

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
