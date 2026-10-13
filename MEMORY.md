# pargen — Memory

Institutional knowledge for the pargen crate (Rust parser generator: ANTLR4 grammar → multi-target parsers).
Use this file to find proven patterns, avoid known pitfalls, and follow established conventions.

---

## 1. AST & parser-of-grammar patterns

- **Grammar AST** lives in `src/grammar/ast.rs`; every node carries `Debug, Clone, PartialEq, Serialize, Deserialize`.
- `Rule::mode: Option<String>` tracks which `mode NAME;` block owns a rule (`None` = default mode). See `grammar::ast::DEFAULT_MODE`.
- `ElementKind::Predicate(String)` and `ElementKind::Action(String)` carry their raw `{ ... }` source. They are parsed but **not emitted** by any codegen (see limitations in ARCHITECTURE.md).
- Antlr-style element kinds `Not(Box<ElementKind>)` wrap an inner kind — generated parsers treat them as no-ops (`src/codegen/rust.rs:639-641`).
- String literals in parser rules are collected and promoted to **implicit tokens** by `analysis::first_follow::collect_implicit_tokens`. The generated lexer gets a `try_match_<T_n>` function for each.
- Left recursion is eliminated *before* codegen via `analysis::left_rec::eliminate_left_recursion`. `ProcessedGrammar::transformed_rules` is what codegen should walk for parser rules; `processed.parser_rules()` chains transformed + untransformed.

## 2. Code generation patterns

- Every target implements `codegen::CodeGenerator` with three methods: `lang_name`, `file_extension`, `generate(&ProcessedGrammar) -> String`.
- Output writes go through `CodeWriter` (indent-aware line builder) — never `format!` directly into a final `String`.
- Naming helpers in `codegen/mod.rs`: `snake_case`, `pascal_case`, `camel_case`, `escape_string`, `collect_tokens` (use these — do not roll your own).
- The Rust target is the only one with a **typed CST enum** (`<Name>Node`). Other targets emit class hierarchies (Go/TS/Python/Java/C/C++) or `grammar.js` (Tree-sitter). Treat them as fundamentally different shapes — don't try to share trait-based walker concepts with them.
- `gen_*` functions follow a consistent signature: `fn gen_<thing>(w: &mut CodeWriter, name: &str, pg: &ProcessedGrammar)`. Add new pieces by following this exact shape and wiring them into `RustGenerator::generate`.

## 3. Visitor / Listener pattern (Rust target) — added 2026-10-13

- **Where**: `src/codegen/rust.rs::gen_visitor`, `gen_listener`, `gen_walker`. Wired in `RustGenerator::generate` between the parser and the error blocks.
- **Shape**:
  - `<Name>Visitor` trait — one `visit_<rule>(&mut self, _node: &<Name>Node) {}` per parser rule + `visit_token(&mut self, _node: &<Name>Node) {}`. Default impls are no-ops so users override only what they need.
  - `<Name>Listener` trait — `enter_<rule>` / `exit_<rule>` per parser rule + `visit_token`. Same default-no-op rule.
  - `walk_<snake_grammar>(visitor: &mut V, node: &<Name>Node)` for visitor DFS — calls `visit_<rule>` *before* recursing into children.
  - `walk_<snake_grammar>_listen(listener: &mut L, node: &<Name>Node)` for listener DFS — brackets children between `enter_<rule>` and `exit_<rule>`.
- **Trait bound trick**: `<V: Visitor + ?Sized>` lets the walker accept trait objects (`Box<dyn Visitor>`) and smart pointers (`Rc<RefCell<…>>`).
- **Method arg**: visitor and listener methods always take `&<Name>Node` — *not* `&<Name>Node::Variant`. Variant types are not directly referable as method parameter types in trait declarations; taking the enum root and pattern-matching in the impl is the idiomatic Rust API.
- **Walker dispatch**: `match node { <Name>Node::rule{ children, .. } => { visit_*(node); walk_(children) }, <Name>Node::Token { .. } => { visit_token(node) } }`. The exhaustive match is guaranteed to compile (compile error if a new variant is added).
- **Tests**:
  - `tests/codegen_test.rs::test_rust_codegen_emits_visitor_trait`, `test_rust_codegen_emits_listener_trait`, `test_rust_codegen_emits_walkers`, `test_rust_codegen_visitor_methods_per_rule`, `test_rust_codegen_no_visitor_in_other_targets` — 5 new tests, all passing.
  - Snapshot `tests/snapshots/codegen_test__snapshot_simple_rust.snap` was refreshed to include the visitor/listener blocks (changes are exactly additive — no other output shifted).
- **End-to-end verification**: built a hand-compiled `Cargo` crate extracting just the visitor section (`/tmp/calc_visitor_check`) with a `Counter` visitor and an `EnterExitLog` listener. Both ran correctly over a synthetic CST. This verified the generated code compiles *in isolation* — the unrelated charset-lexer fix (see §3b) later allowed the full `simple_parser.rs` to compile as well.
- **Scope decision**: visitor/listener is **Rust-only**. Other targets emit language-idiomatic ASTs without a typed enum root, so the same machinery wouldn't translate cleanly. Re-evaluate only if/when another target adopts a typed CST enum.

## 3a. Semantic predicate detection — added 2026-10-13

- **Where**: `src/grammar/ast.rs` — `Grammar::predicates()` + `PredicateInfo` struct, re-exported from `lib.rs`. Surfaced in CLI (`src/main.rs::Commands::Check`) and MCP (`src/mcp.rs::get_grammar_info`).
- **Shape**:
  - `pub struct PredicateInfo { rule_name: String, alternative_index: usize, element_index: usize, source: String }` — derives `Debug, Clone, PartialEq, Serialize, Deserialize` (matches every other AST type).
  - `pub fn Grammar::predicates(&self) -> Vec<PredicateInfo>` — recursive walk through every alternative, descending into `( ... )`, `( ... )?`, `( ... )*`, `( ... )+` groups. Does NOT enter `RuleRef` (a predicate inside a sub-rule is reported by that sub-rule's call, not duplicated here).
- **Reporting-only**: generated parsers do NOT execute predicates for alternative selection — they remain no-ops at runtime. This iteration is detection-only; future enforcement requires a `<Name>PredicateSolver` trait plumbed through every `parse_<rule>` call.
- **Why reporting-only first**: predicate bodies are arbitrary host-language expressions (`$a > 0`, `isInCurrentScope()`, etc.). Even if parsed, they don't translate across targets without per-target solver integration. Reporting gives users and tooling visibility today without locking in a specific execution model.
- **CLI format**: under `pargen check`, predicates print as `Semantic predicates: N (not enforced by codegen yet)` plus one line per predicate with rule / alt-idx / elem-idx / source. Requires `RUST_LOG=info` to be visible (existing project behavior — `fmt::init()` respects env filter).
- **MCP shape**: `get_grammar_info` returns `predicates: [{rule, alternative_index, element_index, source}, ...]` plus a `predicates_count` field. Matches the existing flat structure used for `parser_rules` and `lexer_rules`.
- **Tests**: 7 new tests total — 4 inline in `src/grammar/ast.rs::tests` (empty / top-level / nested-in-group / multi-alternative) and 3 integration in `tests/integration_test.rs` (real lex+parse path, no-predicate grammar, serde round-trip via JSON).
- **Anti-pattern observed**: `Rule`'s struct has 9 fields (`is_fragment`, `modifiers`, `return_type`, `locals_decl`, `throws`, `alternatives`, `commands`, `mode`, `name`). When constructing a `Rule` literal in tests, you must supply *every* field — partial-update helps against `non_exhaustive`-style additions later but slows down test scaffolding. Use a helper `fn parser_rule(name, alts)` rather than spelling the literal out.

## 3b. Charset lexer codegen fix — resolved 2026-10-13

- **Bug (pre-existing)**: Generated Rust lexers emitted `if Self::char_in_ranges(c, &RANGES) { ... }` to test charset membership, but no `char_in_ranges` method or `RANGES` constant was ever defined. Any grammar whose lexer rules used charsets (`[a-z]+`, `[\t\n]+`, etc.) produced code that failed to compile. The original `// charset: [a-z]` comment was the only context ever emitted.
- **Fix** (per Mature Module Policy: surgical, regression test required, no surrounding restructuring):
  1. Added `charset_match_expr(&CharSetDef) -> String` in `src/codegen/rust.rs`.
  2. Replaced the two `Self::char_in_ranges(c, &RANGES)` references with the inline expression produced by that helper.
- **Expr shape**:
  - Multi-char range: `('a'..='z').contains(&c)`.
  - Single-char range: `c == '\t'` (avoids `('\\t'..='\\t').contains(&c)` noise).
  - Multi-range set: `('a'..='z').contains(&c) || ('A'..='Z').contains(&c)` (OR-joined).
  - Empty: `false` (no ranges = matches nothing).
  - Negated: `!(...)` wrapping the OR-joined result.
- **Char-literal escaping** (in `charset_match_expr::escape_char`): `\\` → `'\\\\'`, `'` → `'\''`, `\n`/`\r`/`\t`/`\0` → standard Rust escapes, control chars < 0x20 → `'\u{XX}'`, codepoints > 0x7E → `'\u{XXXX}'`, others → `'X'` as-is.
- **Tests**: 3 regression tests in `tests/codegen_test.rs::test_rust_codegen_charset_*` — no `Self::char_in_ranges`, no `&RANGES`, presence of inline range check (`('a'..='z').contains(&c)`), and zero regressions on a negation-path grammar.
- **Snapshot**: `tests/snapshots/codegen_test__snapshot_simple_rust.snap` refreshed. Diff was exactly two line replacements (one in `try_match_i_d`, one in `try_match_w_s`).
- **End-to-end verify**: built `/tmp/pargen_charset_verify` with the freshly generated `simple_parser.rs` as `lib.rs` — compiles clean with only pre-existing snake-case warnings unrelated to the fix. (Proves the generated lexer is now valid standalone Rust, which it wasn't before.)
- **Did NOT touch**: `gen_lexer_element_match`'s `saved_pos` restore path, `gen_lexer_match_fn`'s `len == 0` return paths, etc. Investigation (2026-10-13) showed these *look* like missing-pos-restore bugs but are **unreachable in practice today**: `peek_at` doesn't advance `self.pos`, and `len == 0` only fires at EOF where `self.pos == input.len()`. Documented in ARCHITECTURE Known Limitations §6 as investigated-and-benign rather than as a real follow-up.

## 3c. Lexer channel detection — added 2026-10-13

- **Where**: `src/grammar/ast.rs` — `Grammar::channels()` + `ChannelInfo` struct, re-exported from `lib.rs`. Surfaced in CLI (`src/main.rs::Commands::Check`) and MCP (`src/mcp.rs::get_grammar_info`). Symmetric with §3a (predicate detection).
- **Shape**:
  - `pub struct ChannelInfo { name: String, assigned_rules: Vec<String> }` — derives `Debug, Clone, PartialEq, Serialize, Deserialize`.
  - `pub fn Grammar::channels(&self) -> Vec<ChannelInfo>` — walks `channel_specs` first (declared channels in declaration order), then appends implicitly-referenced channels in the order each name is first seen via `Rule::channel()`. Mirrors ANTLR's implicit-channel behavior so users get a complete picture.
- **Reporting-only**: generated parsers do NOT route tokens by channel — they remain default-channel for every token. This iteration is detection-only; full channel routing requires:
  1. Adding a `channel` field to `<Name>Token`.
  2. Emitting a `<Name>LexerChannel { Default, HIDDEN, ... }` enum.
  3. Wiring per-rule channel commands into `make_token`.
  4. Filtering hidden-channel tokens out of the parser's input stream (or threading a current-channel parameter through every `parse_<rule>`).
- **Why reporting-only first**: same logic as predicates — channel routing is invasive across all 8 targets (or at least the typed-CST Rust target), and ANTLR's pattern requires parser-side channel awareness. Reporting-first gives users immediate visibility without locking in a parser-API choice.
- **CLI format**: `Lexer channels: N (routing not emitted by codegen yet)` plus per-channel rule list (`HIDDEN ← COMMENT`).
- **MCP shape**: `get_grammar_info` returns `channels: [{name, assigned_rules}, ...]` plus `channels_count`.
- **Tests**: 9 new tests total — 6 inline in `src/grammar/ast.rs::tests` covering declared-only / implicit-only / declared+implicit / multi-rule-to-one / ordering / serde shape, and 3 integration in `tests/integration_test.rs` exercising the real lex+parse path.
- **Hit a pre-existing limit while testing**: the strict lexer slurps `{ HIDDEN, ERROR }` (the `channels { ... }` declaration block) into a single `Action` token — same flaw as `options { ... }`. Per-rule `-> channel(NAME)` works because no `{` follows. The integration tests use `parse_grammar_source_detailed` (which goes through the recovery lexer) to demonstrate the implicit-channel fallback. Documented in ARCHITECTURE.md Known Limitations §7 as a follow-up.
- **Anti-pattern observed (continued from §3a)**: when extending `Rule`'s helper builders in tests, `LexerCommand::channel("HIDDEN")` is the right call — direct construction via `LexerCommand::new("channel", Some("HIDDEN".into()))` is also fine, but using the builder keeps test code symmetric with parser-level code.

## 4. Error handling & diagnostics

- `crate::core::Error` uses `thiserror`; variants: `Io`, `Lexer`, `Parser`, `Analysis`, `Codegen`, `UnsupportedLanguage`, `ValidationFailed`, `Multiple`. Add a new variant via `#[error("...")]` + a builder.
- `Diagnostic::at(line, col)` and `.with_note("...")` produce structured, user-actionable error messages with fix-it hints.
- `ErrorBag` collects many diagnostics; `to_error()` collapses a one-diagnostic bag into a single `Error` for ergonomic single-error paths.
- Two API surfaces:
  - **Strict**: `parse_grammar_file` / `parse_grammar_source` → bail on first error.
  - **Recovery**: `parse_grammar_source_detailed` → `(Option<Grammar>, Vec<Diagnostic>)`. Used by `cli check` and `cli generate` so users see every error per run.

## 5. Testing patterns

- **Inline unit tests** (`#[cfg(test)] mod tests`) live next to the source they exercise — `src/grammar/lexer.rs`, `src/grammar/parser.rs`, `src/analysis/left_rec.rs`, `src/core/error.rs`.
- **Integration tests** at `tests/integration_test.rs` exercise the public API end-to-end (CLI per-subcommand, MCP tools).
- **Codegen tests** at `tests/codegen_test.rs` — structure assertions (`content.contains(...)`) + 8 `insta` snapshots per language. Snapshot updates run via `INSTA_UPDATE=always cargo test`.
- **Proptest** in `src/grammar/lexer.rs` — property-based robustness: never panic on arbitrary input, comment → EOF, grammar header tokens.
- **Test count ledger** is in `ARCHITECTURE.md` and `TODO.md`. Update both when adding/removing tests so the docs stay in sync (current total: **175 passing**).

## 6. CLI / MCP

- `clap` with derive — `src/main.rs::Cli`. Subcommands: `generate`, `init`, `parse`, `check`. Each takes `-g/--grammar <file>` plus a target flag.
- `tracing_subscriber::fmt::init()` at startup → `RUST_LOG=info pargen …` works out of the box. Use `tracing::info!` / `tracing::debug!`, not `println!`.
- MCP server (`src/mcp.rs` + `src/bin/pargen-mcp.rs`) exposes 5 tools over stdio transport: `parse_grammar`, `validate_grammar`, `generate_code`, `get_grammar_info`, `list_target_languages`. Schemas are derived via `schemars` annotations on input structs.
- Codegen failures should bubble up via `anyhow::Result`, then be formatted with `.context(...)`. Don't surface raw `String` errors.

## 7. Benchmarks — added 2026-10-13

- **Where**: `benches/parser_codegen.rs` (criterion target, `harness = false` in `Cargo.toml`) and `tests/bench_smoke.rs` (3 integration tests that exercise the public API the benches depend on).
- **Why both files**: criterion's `criterion_main!` replaces the Rust test harness's `main` for the bench crate, so `#[test]` items inside `benches/parser_codegen.rs` don't run under `cargo test`. Putting the smoke tests in `tests/bench_smoke.rs` gives `cargo test` a way to lock in the public API surface (`parse_grammar_source`, `parse_grammar_source_detailed`, `ProcessedGrammar::process`, `codegen::get_generator`).
- **Why so few targets (4)**: each bench should measure *one* phase. Combining parse + analysis + codegen hides which step regressed. The current four cover the dominant pipeline paths; expanding to all 8 codegen targets is on the brainstorming backlog.
- **Initial numbers** (release profile, --warm-up-time 0.1, --measurement-time 0.2, 2026-10-13, this MacBook): strict parse 4.2 µs, recovery parse 4.1 µs, full Rust pipeline 260 µs (~58× slower than parse — String allocation per line via `CodeWriter` dominates), scaling N=64 11.3 µs.
- **Anti-pattern observed**: bench fixtures (`CALCULATOR_SOURCE`) live in `benches/parser_codegen.rs`; the smoke test's `CALCULATOR_SOURCE` is a separate `const` in `tests/bench_smoke.rs`. They must stay in sync by hand. A `tests/common/mod.rs` or `pub const` re-export would be cleaner but adds a layer for only two short fixtures — not worth it yet. Note in code review if a third fixture appears.

---

## Known bugs & pitfalls

- **Clippy warnings (~73)** are mostly pre-existing style preferences (`useless_format`, `inherent_to_string`). New code is expected to pass clippy clean — write new code without them, even if the surrounding code violates them.
- **`parse` CLI subcommand** is a placeholder — it doesn't yet execute a generated parser on input. That's a deferred feature, not a bug per se.
- **Channel support** (`-> channel(HIDDEN)`) is in the AST (`RuleModifier::Channel`) but only `skip` and mode commands are codegen'd.
- **Position-restore in generated lexer** — investigated (2026-10-13), found to be unreachable today (`peek_at` doesn't advance `self.pos`; `len == 0` only fires at EOF). Documented in ARCHITECTURE Known Limitations §6 as benign. Do NOT apply a defensive fix without a concrete reproducer (per Mature Module Policy).
- **`channels { HIDDEN }` declaration block** — strict lexer slurps `{ HIDDEN }` into a single Action token, so the `channels { ... }` block is unparseable via `parse_grammar_source` (only via `parse_grammar_source_detailed`). Per-rule `-> channel(NAME)` works because no `{` follows. See ARCHITECTURE Known Limitations §7.

## When MEMORY.md needs attention

- After completing any non-trivial feature or bug fix
- When discovering a new pattern or anti-pattern
- After resolving a tricky debugging session (especially when it touches ANTLR semantics or codegen oddities)
- When establishing new conventions (e.g. a new generator helper)
- When a known reference test value is confirmed

Format: be specific (file path + line range), include minimal code snippets, explain *why* not just *what*. Cross-reference related patterns via `[[Pattern Name]]`.

Last updated: 2026-10-13
