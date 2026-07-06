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

- Full ANTLR4 feature parity (visitor/listener, channels, lexer modes, semantic predicates).
- Incremental or error-tolerant parsing in generated parsers.
- Building a full compiler pipeline (type checking, optimization, code emission beyond parsers).

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

Each target implements the `CodeGenerator` trait. Output is written to `{grammar_name}_parser{extension}` in the specified directory.

## Quality Bar

- All changes must pass `cargo test` (currently 116 tests).
- New codegen targets require integration tests in `tests/`.
- New lexer/parser behavior requires unit tests alongside the component.
- Public API changes must be reflected in `README.md` and `ARCHITECTURE.md`.
- Structured errors must include source location (line, column) where applicable.

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

| Language | CLI flag | Output |
|----------|----------|--------|
| Rust | `rust` | `.rs` |
| Go | `go` | `.go` |
| TypeScript | `typescript`, `ts` | `.ts` |
| Python | `python`, `py` | `.py` |
| Java | `java` | `.java` |
| C | `c` | `.c` |
| C++ | `cpp`, `c++`, `cplusplus` | `.cpp` |
| Tree-sitter | `treesitter`, `tree-sitter` | `grammar.js` |

---

Last updated: 2026-07-06
