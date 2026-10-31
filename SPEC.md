# pargen — Specification

## Scope

`pargen` is a standalone parser generator that reads ANTLR4 grammar files (`.g4`) and emits self-contained parsers or Tree-sitter grammars. It targets developers who already have ANTLR4 grammars and want generated parsers without a Java runtime or ANTLR toolchain.

## Goals

1. **ANTLR4 grammar compatibility** — Accept standard `.g4` syntax for lexer and parser rules.
2. **Multi-target code generation** — Emit parsers in Rust, Go, TypeScript, Python, Java, C, C++, and Tree-sitter `grammar.js`.
3. **Zero runtime in generated code** — Generated parsers must not depend on `pargen` or ANTLR at runtime.
4. **Programmatic API** — Expose `parse_grammar_file`, `parse_grammar_source`, and `generate` for library use.
5. **MCP integration** — Provide a `pargen-mcp` binary for AI tooling (parse, validate, generate, inspect).

## Non-Goals

- Full ANTLR4 feature parity. Specifically: semantic predicate *enforcement* and lexer *channel routing* are detected and reported, but not executed at runtime.
- Incremental or error-tolerant parsing in generated parsers.
- Building a full compiler pipeline (type checking, optimization, code emission beyond parsers).
- Visitor/listener generation for the non-Rust targets (the Rust target emits them; see below).

### Supported Subsets

| Feature | Status |
|---|---|
| Lexer modes (`mode NAME;`) | Implemented in the Rust target; the other 7 targets parse modes but treat all rules as default-mode. |
| Visitor / listener traits | Implemented in the Rust target only. |
| Semantic predicates (`{ ... }?`) | Detection + reporting only. Not enforced at runtime in any target. |
| Lexer channels (`-> channel(NAME)`) | Detection + reporting only. Tokens are not routed by channel in any target. |
| Grammar `import` / `tokenVocab` | Parsed but not merged across files. |

## Technical Stack

| Layer | Technology |
|-------|------------|
| Language | Rust 2024 (MSRV 1.85) |
| CLI | `clap` |
| Errors | `thiserror` (library), `anyhow` (CLI/high-level API) |
| Logging | `tracing` + `tracing-subscriber` |
| Serialization | `serde` + `serde_json` |
| MCP server | `rmcp` + `tokio` + `schemars` |
| Testing | `cargo test`, `insta`, `proptest`, `tempfile`, `criterion` |

## Pipeline

```
.g4 file → Lexer → Parser → AST → Analysis → CodeGen → output file
```

### Analysis Phase

- Compute FIRST/FOLLOW sets
- Eliminate direct and indirect left recursion
- Promote implicit string-literal tokens

### Code Generation

Each target implements the `codegen::CodeGenerator` trait. Output is written to `{grammar_name_lowercase}_parser{extension}` in the specified directory.

| Language | CLI flag | Output extension |
|----------|----------|------------------|
| Rust | `rust` | `.rs` |
| Go | `go` | `.go` |
| TypeScript | `typescript`, `ts` | `.ts` |
| Python | `python`, `py` | `.py` |
| Java | `java` | `.java` |
| C | `c` | `.h` |
| C++ | `cpp`, `c++`, `cplusplus` | `.hpp` |
| Tree-sitter | `treesitter`, `tree-sitter` | `.js` |

The C and C++ targets emit a single header-only file containing both the lexer and the parser. The Tree-sitter target emits a `grammar.js` module (named `{grammar}_parser.js`).

## Quality Bar

- All changes must pass `cargo test` (currently 175 tests).
- New codegen targets require integration tests in `tests/`.
- New lexer/parser behavior requires unit tests alongside the component.
- Public API changes must be reflected in `README.md` and `ARCHITECTURE.md`.
- Structured errors must include source location (line, column) where applicable.
- Diagnostics may include a `note` for fix-it hints.
- The test-count ledger is maintained in both `ARCHITECTURE.md` and `TODO.md`.

## CLI Commands

| Command | Description |
|---------|-------------|
| `generate -g <file> -l <lang> -o <dir>` | Parse grammar and emit target-language parser |
| `check -g <file>` | Validate grammar and report rule counts |
| `init -n <name> -o <dir>` | Scaffold a minimal `.g4` file |
| `parse -g <file> -i <input>` | Placeholder — requires generated parser |

## MCP Tools (`pargen-mcp`)

| Tool | Purpose |
|------|---------|
| `parse_grammar` | Parse grammar text, return AST summary |
| `validate_grammar` | Parse + analyze, return diagnostics |
| `generate_code` | Full pipeline to target language |
| `get_grammar_info` | Detailed rule lists and options |
| `list_target_languages` | List all 8 supported targets |

## Supported Targets

See the Code Generation table above for the authoritative flag and extension mapping.

---

Last updated: 2026-10-31
