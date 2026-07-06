# pargen

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/yingkitw/pargen/blob/main/Cargo.toml)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-orange.svg)](https://doc.rust-lang.org/edition-guide/editions/overview.html)
[![GitHub](https://img.shields.io/badge/github-yingkitw%2Fpargen-181717.svg?logo=github)](https://github.com/yingkitw/pargen)

A parser generator using ANTLR4 grammar syntax, outputting standalone parsers for Rust, Go, TypeScript, Python, Java, C, C++, and Tree-sitter.

## Why Pargen?

You already have ANTLR4 grammars. Rewriting them for every target language — or dragging a Java runtime into every build — is friction you don't need.

**Pargen lets you keep your `.g4` files and ship parsers anywhere.**

| | ANTLR4 | LALRPOP / Pest | Pargen |
|---|---|---|---|
| Grammar format | `.g4` (ANTLR4) | Language-specific | `.g4` (ANTLR4) |
| Toolchain | Java + ANTLR jar | Rust only | Single Rust binary |
| Generated runtime | ANTLR runtime per language | None | None |
| Multi-target from one grammar | Yes (with runtime) | No (Rust only) | Yes (8 targets) |
| Tree-sitter export | No | No | Yes |

**Use Pargen when you:**

- Have existing `.g4` grammars and want parsers in Rust, Go, TypeScript, or other languages without the ANTLR runtime
- Need self-contained generated code you can drop into any project — no generated-code dependency on Pargen or ANTLR
- Want one grammar to feed both a compiler/interpreter *and* a Tree-sitter grammar for editor tooling
- Prefer a lightweight, `cargo install`-able tool over a Java-based code-generation pipeline
- Want MCP integration so AI assistants can parse, validate, and generate parsers from grammar text

**Reach for something else when you:**

- Are starting from scratch in Rust only — [LALRPOP](https://github.com/lalrpop/lalrpop) or [Pest](https://github.com/pest-parser/pest) may be a better fit
- Need full ANTLR4 features (visitor/listener, lexer modes, semantic predicates) — Pargen does not aim for full parity yet
- Need incremental, error-tolerant parsing for a live editor — generate a Tree-sitter grammar with Pargen, then use Tree-sitter's runtime

## Features

- **ANTLR4-compatible grammar syntax** — Use existing `.g4` grammar files
- **Multi-language code generation** — Generate parsers in 8 target languages
- **Tree-sitter export** — Emit `grammar.js` for editor tooling
- **No runtime dependencies** — Generated parsers are self-contained
- **Semantic analysis** — First/follow sets, left-recursion elimination
- **Structured errors** — Source locations on lexer/parser failures
- **MCP server** — `pargen-mcp` binary with 5 tools for AI integration
- **Structured logging** — `tracing`-based output with `RUST_LOG` support

## Supported Target Languages

| Language | Status | Extension |
|---|---|---|
| Rust | Supported | `.rs` |
| Go | Supported | `.go` |
| TypeScript | Supported | `.ts` |
| Python | Supported | `.py` |
| Java | Supported | `.java` |
| C | Supported | `.c/.h` |
| C++ | Supported | `.cpp/.hpp` |
| Tree-sitter | Supported | `grammar.js` |

## Installation

```bash
cargo install --path .
```

## Usage

### Generate a Parser

```bash
pargen generate -g grammar.g4 -l rust -o output/
pargen generate -g grammar.g4 -l go -o output/
pargen generate -g grammar.g4 -l typescript -o output/
pargen generate -g grammar.g4 -l treesitter -o output/
```

### Validate a Grammar

```bash
pargen check -g grammar.g4
```

### Initialize a New Grammar

```bash
pargen init -n MyGrammar -o ./myproject
```

### MCP Server

```bash
pargen-mcp   # stdio transport for MCP clients
```

## Example Grammar

```antlr
grammar Calculator;

expr: term (('+' | '-') term)*;
term: factor (('*' | '/') factor)*;
factor: NUMBER | '(' expr ')';

NUMBER: [0-9]+;
WS: [ \t\r\n]+ -> skip;
```

## Architecture

```
.g4 → Lexer → Parser → AST → Analysis → CodeGen → output
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for module layout and extension points.

## Development

```bash
cargo build
cargo test          # 116 tests
RUST_LOG=debug cargo run -- generate -g examples/calc.g4 -l rust
```

## License

Apache-2.0
