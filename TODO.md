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

**Test Summary**: 108 tests passing
- 28 lexer unit tests (includes 4 proptests)
- 27 parser unit tests
- 8 left-recursion unit tests
- 13 core error unit tests
- 22 integration tests (8 languages)
- 10 codegen tests

---
Last updated: 2026-04-27
