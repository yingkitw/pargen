# pargen

A parser generator using ANTLR4 grammar syntax, outputting standalone parsers for Rust, Go, TypeScript, Python, Java, C, C++, and Tree-sitter.

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
