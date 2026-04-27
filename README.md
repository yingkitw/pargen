# pargen

A parser generator using ANTLR4 grammar syntax, outputting standalone parsers for Rust, Go, TypeScript, Python, Java, C, and C++.

## Features

- **ANTLR4-compatible grammar syntax** — Use existing `.g4` grammar files
- **Multi-language code generation** — Generate parsers in 7 target languages
- **No runtime dependencies** — Generated parsers are self-contained
- **Semantic analysis** — First/follow sets, left-recursion elimination
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
```

### Validate a Grammar

```bash
pargen check -g grammar.g4
```

### Initialize a New Grammar

```bash
pargen init -n MyGrammar -o ./myproject
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

## Development

```bash
cargo build
cargo test
RUST_LOG=debug cargo run -- generate -g examples/calc.g4 -l rust
```

## License

Apache-2.0
