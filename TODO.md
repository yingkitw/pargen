# pargen - Task List

## Learnings from minipg - High Priority

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

### [ ] 7. Create `ARCHITECTURE.md`
- Module overview (grammar, analysis, codegen)
- Data flow: grammar file -> lexer -> parser -> AST -> analysis -> codegen
- Extension points (new target languages)

## Learnings from minipg - Medium Priority

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

## Learnings from minipg - Low Priority / Future

### [ ] 11. MCP Server Integration
- Add `rmcp` + `tokio` + `schemars` dependencies
- Create `src/mcp.rs` with tools: `parse_grammar`, `validate_grammar`, `generate_code`
- Add `pargen-mcp` binary

### [ ] 12. Property-Based Testing
- Add `proptest` for robustness testing of lexer/parser

### [ ] 13. Tree-sitter Grammar Generator
- Add `treesitter` target to codegen
- Generate `grammar.js` from ANTLR4 grammar

### [ ] 14. Error Diagnostics with Locations
- Add line/column tracking to lexer tokens
- Rich error messages with source location

## Completed

- [x] 1. Upgrade to Rust Edition 2024
- [x] 2. Add Core Dependencies (serde, thiserror, tracing, insta, criterion, tempfile)
- [x] 3. Create `core` module with Error, Diagnostic, GrammarParser, SemanticAnalyzer, CodeGenerator traits
- [x] 4. Add `Serialize`/`Deserialize` derives to all AST types (Grammar, Rule, Alternative, Element, ElementKind, etc.)
- [x] 5. Refactor `lib.rs` with re-exports for discoverable public API
- [x] 6. Create `README.md` with install, usage, and example
- [x] 8. Replace all `println!` with `tracing` macros + `tracing_subscriber::fmt::init()`
- [x] 9. Add `insta`, `criterion`, `tempfile` to dev-dependencies
- [x] 10. Add release profile optimization (opt-level 3, lto, codegen-units 1, strip)
- [x] 11. Create `.gitignore` with Rust/Cargo, IDE, snapshot, and generated parser artifacts
- [x] 12. Comprehensive test suite (102 tests)
  - 24 lexer unit tests (token kinds, string literals, charsets, comments, errors, locations)
  - 27 parser unit tests (grammar headers, rules, alternatives, groups, labels, actions, fragments, errors)
  - 8 left-recursion unit tests (direct, indirect, empty alternatives, multiple alternatives)
  - 13 core error unit tests (display, serialization, clone/equality)
  - 21 integration tests (end-to-end parse, generate for all 7 languages, error cases)
  - 9 codegen tests (structure verification for all target languages)

---
Last updated: 2026-04-27
