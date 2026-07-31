# pargen - Task List

## High Priority

### [x] 1. Upgrade to Rust Edition 2024
- Update `Cargo.toml`: `edition = "2024"`, `rust-version = "1.85"`
- Fix any compile errors from edition migration

### [x] 2. Add Core Dependencies
- `serde` + `serde_json` — AST serialization, future MCP integration
- `thiserror` — Structured error types (replace raw `anyhow::anyhow!` strings)
- `tracing` + `tracing-subscriber` — Replace `println!` with structured logging

### [x] 3. Create `core` Module with Traits
- `GrammarParser` trait: `parse_file`, `parse_string`
- `SemanticAnalyzer` trait: `analyze`, `diagnostics`
- `CodeGenerator` trait: `generate`, `target_language`
- Unified `Error` / `Result` types with `thiserror`

### [x] 4. Add Serde Serialization to AST
- Add `Serialize` / `Deserialize` derives to `Grammar`, `Rule`, `Alternative`, `Element`, etc.
- Enables programmatic inspection and future MCP tooling

### [x] 5. Refactor `lib.rs` with Re-Exports
- Expose `Grammar`, `Rule`, etc. at crate root
- Expose traits from `core` at crate root
- Make public API discoverable without deep module paths

### [x] 6. Create `README.md`
- Project description, install, usage, supported languages
- Badges, example grammar, CLI commands

### [x] 7. Create `ARCHITECTURE.md`
- Module overview (grammar, analysis, codegen)
- Data flow: grammar file -> lexer -> parser -> AST -> analysis -> codegen
- Extension points (new target languages)

## Medium Priority

### [x] 8. Add Logging with `tracing`
- Replace all `println!` in `main.rs` and `codegen` with `tracing::info!` / `tracing::debug!`
- Add `RUST_LOG` support via `tracing-subscriber`

### [x] 9. Add Dev Dependencies
- `insta` — Snapshot testing for codegen output
- `criterion` — Benchmarks for parser/codegen performance
- `tempfile` — Integration tests

### [x] 10. Add Release Profile Optimization
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

## Low Priority / Future

### [x] 11. MCP Server Integration
- Added `rmcp` + `tokio` + `schemars` dependencies
- Created `src/mcp.rs` with 5 tools:
  - `parse_grammar` — parse grammar text and return AST summary (rules count, lexer/parser rule counts, options)
  - `validate_grammar` — parse + analyze grammar, return success/failure + diagnostics
  - `generate_code` — parse + analyze + generate code for any supported target language, return generated code
  - `get_grammar_info` — detailed rule lists (parser rules, lexer rules, counts, options)
  - `list_target_languages` — list all 8 supported languages with descriptions
- Added `pargen-mcp` binary with `tokio::main` and `stdio` transport

### [x] 12. Property-Based Testing
- Added `proptest = "1.6"` dev dependency
- Property tests in lexer: never panics, whitespace→EOF, comment→EOF, grammar header tokens

### [x] 13. Tree-sitter Grammar Generator
- Added `treesitter` codegen target
- Generates `grammar.js` with `choice`, `seq`, `optional`, `repeat`, `repeat1`, `token`, regex charsets

### [x] 14. Error Diagnostics with Locations
- Lexer tokens carry `(line, column, offset)`
- `core::Error::lexer()` / `core::Error::parser()` create structured errors with `SourceLocation`
- `G4Lexer::tokenize()` and `G4Parser::parse()` now return `crate::core::Result<>` with proper error types
- `lib.rs` uses `?` propagation directly without `map_err` wrappers

### [x] 15. Error Recovery & Multi-Error Reporting
- `Diagnostic` now has an optional `note` field for fix-it hints
- `ErrorBag` collects multiple `Diagnostic`s; `Error::Multiple { diagnostics }` wraps a bag
- `G4Lexer::tokenize_recover()` returns `(Vec<G4Token>, Vec<Diagnostic>)` — skips unexpected chars, emits EOF on unterminated tokens
- `G4Parser::parse_recover()` returns `(Option<Grammar>, Vec<Diagnostic>)` — syncs to next `;` after each failed rule
- `parse_grammar_source_detailed(source)` is the public multi-error entry point
- CLI `check` and `generate` print every diagnostic with severity + location + note
- MCP `validate_grammar` returns the full `diagnostics` array (not just the first error)

### [x] 16. Lexer Modes Code Generation
- `Rule::mode: Option<String>` tracks which mode each rule belongs to
- `Grammar::lexer_modes()` lists all modes (default + named, sorted)
- `Grammar::lexer_rules_in_mode(mode)` groups rules by mode
- `LexerCommand::mode_target()` returns `ModeTarget::Named` or `ModeTarget::Default`
- Parser tracks `current_mode` across `mode NAME;` blocks; recovery version handles partial mode declarations
- Rust target emits a `LexerMode` enum, a `mode` field on the lexer, and per-mode `try_match_mode_<name>` dispatch functions
- `pushMode(NAME)`, `popMode`, `mode(NAME)` translate to `self.mode = LexerMode::NAME` after a token matches
- Other targets still treat all rules as default-mode (channel support still un-emitted)

## Completed

- [x] 1. Upgrade to Rust Edition 2024
- [x] 2. Add Core Dependencies (serde, thiserror, tracing, insta, criterion, tempfile)
- [x] 3. Create `core` module with Error, Diagnostic, GrammarParser, SemanticAnalyzer, CodeGenerator traits
- [x] 4. Add `Serialize`/`Deserialize` derives to all AST types
- [x] 5. Refactor `lib.rs` with re-exports for discoverable public API
- [x] 6. Create `README.md` with install, usage, and example
- [x] 7. Create `ARCHITECTURE.md` with module overview and data flow
- [x] 8. Replace all `println!` with `tracing` macros + `tracing_subscriber::fmt::init()`
- [x] 9. Add `insta`, `criterion`, `tempfile`, `proptest` to dev-dependencies
- [x] 10. Add release profile optimization (opt-level 3, lto, codegen-units 1, strip)
- [x] 11. MCP server integration (`pargen-mcp` binary with 5 tools)
- [x] 12. Property-based testing with `proptest` (lexer robustness)
- [x] 13. Tree-sitter grammar generator (`treesitter` codegen target)
- [x] 14. Error Diagnostics with Locations (lexer/parser return `crate::core::Result`)
- [x] 15. Error Recovery & Multi-Error Reporting (ErrorBag, tokenize_recover, parse_recover, parse_grammar_source_detailed)
- [x] 16. Lexer Modes Code Generation (Rule::mode, Grammar::lexer_modes(), Rust mode-aware LexerMode dispatch)

**Test Summary**: 148 tests passing (was 137; +11 for lexer modes)
- 31 lexer unit tests (includes 4 proptests + 4 recovery)
- 39 parser unit tests (includes 5 recovery + 5 mode parsing)
- 7 left-recursion unit tests
- 20 core error unit tests (includes ErrorBag + Diagnostic builders)
- 30 integration tests (8 languages + 5 detailed parsing + 3 mode helpers)
- 21 codegen tests (10 structure + 8 insta snapshots + 3 mode codegen)

---

## Brainstorming (Competitive Intelligence)

Compared against ANTLR4, LALRPOP, Pest, and Tree-sitter. Prioritized by competitive advantage for `pargen`'s niche (ANTLR4 grammar → multi-target standalone parsers).

### High Value

- [x] **Error recovery & multi-error reporting** — Added `Diagnostic::note`, `ErrorBag`, `Error::Multiple`, `tokenize_recover`, `parse_recover`, and `parse_grammar_source_detailed`. Parser syncs to next `;` after a failed rule. CLI `check`/`generate` and MCP `validate_grammar` print every diagnostic with severity + location + note.
- [x] **Codegen snapshot tests with `insta`** — 8 snapshot tests lock generated output per language for the simple grammar.
- [x] **Lexer modes code generation** — Added `Rule::mode`, `Grammar::lexer_modes()`, `Grammar::lexer_rules_in_mode()`, `LexerCommand::mode_target()`. Parser tracks `current_mode` across `mode NAME;` blocks. Rust target emits a `LexerMode` enum, a `mode` field, and per-mode dispatch functions with `pushMode`/`popMode`/`mode` command handling. Tests cover mode parsing, default vs. named mode assignment, and codegen output.
- [ ] **Semantic predicate enforcement** — `{ ... }?` / `{ ... }^` predicates are in AST but ignored by codegen. ANTLR's key differentiator for context-sensitive grammars.
- [ ] **Visitor / listener pattern generation** — ANTLR auto-generates tree walkers. High demand for compiler pipelines built on generated parsers.

### Medium Value

- [ ] **Grammar import resolution** — ANTLR `import` and `tokenVocab` are parsed but not merged. Blocks composing grammars from libraries.
- [ ] **Channel support in generated lexers** — ANTLR channels (e.g., `HIDDEN`) for comments/whitespace. Parsed but not codegen'd.
- [ ] **Benchmarks with `criterion`** — Dependency present but no benches. Compare parse/codegen throughput vs hand-written parsers.
- [ ] **WASM codegen target** — Enable browser-based grammar playgrounds (competitive with Ohm.js / Peggy).
- [ ] **Swift / C# codegen targets** — ANTLR's strongest ecosystems beyond what pargen already covers.
- [ ] **Grammar diff / migration tool** — Convert between ANTLR4 and Tree-sitter grammars bidirectionally (pargen already does ANTLR → Tree-sitter one-way).

### Lower Priority / Exploratory

- [ ] **Incremental parsing** — Tree-sitter's core advantage for editors. Requires GLR/incremental runtime, not just grammar.js export.
- [ ] **Tree-sitter query/highlight generation** — Emit `.scm` query files alongside `grammar.js` for full editor integration.
- [ ] **Grammar visualization** — ANTLR plugins show railroad diagrams. Useful for documentation and grammar debugging.
- [ ] **Online playground** — Web UI for paste-grammar-generate (Ohm.js, Peggy offer this).
- [ ] **LALRPOP-style grammar macros** — Parameterized rule templates for DRY grammars (comma-separated lists, etc.).
- [ ] **AST auto-generation** — ANTLR and LALRPOP build typed AST nodes; pargen generates parse functions only.

---

Last updated: 2026-07-31
