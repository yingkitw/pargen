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

### [x] 17. Visitor / Listener Tree-Walker Generation (Rust target)
- Emits `<Name>Visitor` trait with `visit_<rule>(&mut self, _node: &<Name>Node)` per parser rule + `visit_token`. Default impls are no-ops; override what you need.
- Emits `<Name>Listener` trait with `enter_<rule>` / `exit_<rule>` per parser rule + `visit_token`. Default impls are no-ops.
- Emits `walk_<snake_grammar_name>(visitor, &node)` for visitor DFS and `walk_<snake_grammar_name>_listen(listener, &node)` for listener DFS.
- Operates on the existing `<Name>Node` CST — no grammar changes required.
- Scope-limited to Rust target in this iteration; other targets emit language-idiomatic CSTs without walkers (deferred, see brainstorming).
- Tests: 5 new tests in `tests/codegen_test.rs` lock the trait shapes, per-rule methods, walker names, and Rust-only scope. Snapshot updated for the simple grammar. Verified end-to-end with a hand-built `Cargo` verify crate that compiles and runs 2 walker tests.

### [x] 18. Semantic Predicate Detection (analysis + reporting only)
- Add `Grammar::predicates() -> Vec<PredicateInfo>` + `PredicateInfo { rule_name, alternative_index, element_index, source }` to `src/grammar/ast.rs`.
- Re-export `PredicateInfo` from `lib.rs`.
- CLI `check` reports the count of `Semantic predicates: N`.
- MCP `get_grammar_info` returns a `predicates` array (rule, alt, idx, source).
- **No changes** to generated parser code — predicates remain no-ops at runtime. This is detection-only, exposing information users and tooling can build on. Full alternative-disambiguating predicate execution is deferred to a later iteration (it would require threading a predicate solver through every generated `parse_<rule>` invocation).
- Tests: inline unit tests in `ast.rs` for the helper; integration test counting predicates on the existing `test_parse_predicate` grammar shape.

### [x] 19. Fix undefined `RANGES`/`char_in_ranges` in Rust lexer codegen
- **Bug**: generated lexers referenced `Self::char_in_ranges(c, &RANGES)` and `&RANGES`, but neither was ever defined. Any grammar that used a charset (`[a-z]`, `[\t\n]`, etc.) in a lexer rule produced code that failed to compile.
- **Fix**: replaced the broken refs with an inline boolean expression built from each `CharSetDef`'s ranges via the new `charset_match_expr` helper in `src/codegen/rust.rs`. Char ranges are emitted as `('a'..='z').contains(&c)`, single-char ranges as `c == '\t'`, OR-combined for multi-range sets, and wrapped in `!{...}` for negated charsets.
- **Scope**: surgical 2-line replacement + the new ~40-line helper. Per Mature Module Policy (rust.rs is a mature file) no surrounding code was touched.
- **Snapshot**: `tests/snapshots/codegen_test__snapshot_simple_rust.snap` refreshed to show `('a'..='z').contains(&c)` and `c == '\t' || c == '\n'` for the simple grammar's `ID: [a-z]+` and `WS: [\t\n]+`.
- **Tests**: 3 new regression tests in `tests/codegen_test.rs::test_rust_codegen_charset_*` lock the absence of undefined refs, presence of inline range checks, and behavior under negation.

## Completed
- `Rule::mode: Option<String>` tracks which mode each rule belongs to
- `Grammar::lexer_modes()` lists all modes (default + named, sorted)
- `Grammar::lexer_rules_in_mode(mode)` groups rules by mode
- `LexerCommand::mode_target()` returns `ModeTarget::Named` or `ModeTarget::Default`
- Parser tracks `current_mode` across `mode NAME;` blocks; recovery version handles partial mode declarations
- Rust target emits a `LexerMode` enum, a `mode` field on the lexer, and per-mode `try_match_mode_<name>` dispatch functions
- `pushMode(NAME)`, `popMode`, `mode(NAME)` translate to `self.mode = LexerMode::NAME` after a token matches
- Other targets still treat all rules as default-mode (channel support still un-emitted)

### [x] 20. Lexer Channel Detection (analysis + reporting only)
- Add `Grammar::channels() -> Vec<ChannelInfo>` plus `ChannelInfo { name, assigned_rules: Vec<String> }` to `src/grammar/ast.rs`. Reports declared channels (`Grammar::channel_specs`) and the parser-lexer rules that use `-> channel(NAME)`. Mirrors item 18's predicate-detection pattern.
- Re-export `ChannelInfo` from `lib.rs`.
- CLI `check` reports `Lexer channels: N` plus per-channel rule list.
- MCP `get_grammar_info` adds a `channels` array (name + rule list).
- **No codegen change** — same as item 18: detection-first unblocks tooling and human inspection. Full channel routing (default vs hidden filtering, parser-side channel selection) is deferred because it requires touching every generated `parse_<rule>` plus the lexer's token-emit loop.
- Tests: inline unit tests in `ast.rs` for the helper; integration test counting channels + assigning rules for a grammar that uses `channels { HIDDEN, ERROR }` plus `COMMENT: ... -> channel(HIDDEN);`.

### [x] 21. Criterion Benchmarks (baseline)
- Added `benches/parser_codegen.rs` (4 targets via `criterion_group!`).
- Wired via `[[bench]] name = "parser_codegen", harness = false` in `Cargo.toml`.
- Initial quick-mode numbers (release profile, --warm-up-time 0.1, --measurement-time 0.2):
  - `parse_grammar_source_strict`: ~4.2 µs
  - `parse_grammar_source_detailed`: ~4.1 µs
  - `codegen_rust_full_pipeline`: ~260 µs (~58× slower than parse — String allocation per line is the dominant cost)
  - `parse_grammar_source_scaling_n64`: ~11.3 µs (synthetic N=64 implicit-token grammar)
- 3 smoke tests in `tests/bench_smoke.rs` exercise the same public API the benches call. Run via `cargo test --lib` (integration suite); the benchmark harness itself is invoked via `cargo bench`.

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
- [x] 17. Visitor / Listener Tree-Walker Generation (Rust target only). Emits `<Name>Visitor`, `<Name>Listener`, `walk_<snake>(visitor, &node)`, `walk_<snake>_listen(listener, &node)`. 5 new codegen tests + snapshot refresh. End-to-end verified with a hand-compiled `Cargo` verify crate (2 tests run the walker over a synthetic CST).
- [x] 18. Semantic Predicate Detection (reporting-only). Added `Grammar::predicates() -> Vec<PredicateInfo>` (recursive walk through nested groups), `PredicateInfo { rule_name, alternative_index, element_index, source }` re-exported from `lib.rs`. CLI `check` prints `Semantic predicates: N` followed by per-predicate details. MCP `get_grammar_info` returns `predicates` array. No codegen changes — predicates remain no-ops at runtime. 7 new tests (4 inline in `src/grammar/ast.rs` + 3 integration covering top-level, nested-in-group, multi-alternative, no-predicate, and serde round-trip).
- [x] 19. Rust Lexer Codegen: Fix Undefined `RANGES` / `char_in_ranges` References. Pre-existing bug from MEMORY.md. Generated lexers for grammars with charset rules (`[a-z]+`, `[\t\n]+`, etc.) emitted references to `Self::char_in_ranges` and `&RANGES` — neither of which was ever defined. Replaced with inline range checks via new `charset_match_expr` helper. 3 regression tests + snapshot refresh.
- [x] 20. Lexer Channel Detection (reporting-only). Added `Grammar::channels() -> Vec<ChannelInfo>` plus `ChannelInfo { name, assigned_rules: Vec<String> }`. Surfaces both declared (`channels { HIDDEN }`) and implicitly-referenced channels in `-> channel(NAME)` order. CLI `check` prints `Lexer channels: N (routing not emitted by codegen yet)` plus per-channel rule list. MCP `get_grammar_info` adds `channels` array. 9 new tests (6 inline in `src/grammar/ast.rs` + 3 integration) covering declared-only, implicit-only, declared+implicit, multiple-rules-to-one-channel, ordering, and serde round-trip. Discovered that the strict lexer still slurps `{ HIDDEN }` as a single Action token (a pre-existing limitation blocking `channels { ... }` declaration parsing); integration test uses `parse_grammar_source_detailed` for the recovery path and demonstrates the implicit-channel fallback.

**Test Summary**: 175 tests passing (was 172; +3 for bench smoke)
- 31 lexer unit tests (includes 4 proptests + 4 recovery)
- 39 parser unit tests (includes 5 recovery + 5 mode parsing)
- 7 left-recursion unit tests
- 20 core error unit tests (unchanged)
- 10 ast unit tests (predicate + channel detection coverage)
- 39 integration tests (was 36; +3 for bench smoke — exercises the public API the benches depend on)
- 29 codegen tests (10 structure + 8 insta snapshots + 3 mode codegen + 5 visitor/listener + 3 charset regression)

---

## Brainstorming (Competitive Intelligence)

Compared against ANTLR4, LALRPOP, Pest, and Tree-sitter. Prioritized by competitive advantage for `pargen`'s niche (ANTLR4 grammar → multi-target standalone parsers).

### High Value

- [x] **Error recovery & multi-error reporting** — Added `Diagnostic::note`, `ErrorBag`, `Error::Multiple`, `tokenize_recover`, `parse_recover`, and `parse_grammar_source_detailed`. Parser syncs to next `;` after a failed rule. CLI `check`/`generate` and MCP `validate_grammar` print every diagnostic with severity + location + note.
- [x] **Codegen snapshot tests with `insta`** — 8 snapshot tests lock generated output per language for the simple grammar.
- [x] **Lexer modes code generation** — Added `Rule::mode`, `Grammar::lexer_modes()`, `Grammar::lexer_rules_in_mode()`, `LexerCommand::mode_target()`. Parser tracks `current_mode` across `mode NAME;` blocks. Rust target emits a `LexerMode` enum, a `mode` field, and per-mode dispatch functions with `pushMode`/`popMode`/`mode` command handling. Tests cover mode parsing, default vs. named mode assignment, and codegen output.
- [ ] **Semantic predicate enforcement** — Reporting-only done (grammar-level detection in `Grammar::predicates()` + CLI/MCP surfacing). Full alternative-disambiguating enforcement still deferred — would require threading a predicate solver through every generated `parse_<rule>` call.
- [x] **Visitor / listener pattern generation** — Done (Rust target). Emits `<Name>Visitor` with one method per rule + `visit_token`, `<Name>Listener` with `enter_<rule>`/`exit_<rule>` per rule + `visit_token`, plus `walk_<snake>` and `walk_<snake>_listen` DFS helpers. Operates on the typed `<Name>Node` CST — no grammar changes required. Deferring other-target visitors until at least one non-Rust target grows a typed CST enum.

### Medium Value

- [ ] **Grammar import resolution** — ANTLR `import` and `tokenVocab` are parsed but not merged. Blocks composing grammars from libraries.
- [ ] **Channel support in generated lexers** — Reporting-only done (grammar-level detection in `Grammar::channels()` + CLI/MCP surfacing). Full routing (per-channel token filtering, parser-side channel awareness) still deferred — invasive across all 8 targets or at least the typed-CST Rust target.
- [ ] **Benchmarks with `criterion`** — Initial baseline done (4 criterion targets in `benches/parser_codegen.rs`). Re-run with hand-written parsers (or `pest`/`lalrpop` references) for competitive numbers; add memory benches; expand to all 8 codegen targets, not just Rust.
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
