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
- Need full ANTLR4 feature parity (semantic predicate enforcement, lexer channels) — Pargen detects and reports these but does not yet enforce them at runtime
- Need incremental, error-tolerant parsing for a live editor — generate a Tree-sitter grammar with Pargen, then use Tree-sitter's runtime

## Features

- **ANTLR4-compatible grammar syntax** — Use existing `.g4` grammar files
- **Multi-language code generation** — Generate parsers in 8 target languages
- **Tree-sitter export** — Emit `grammar.js` for editor tooling
- **No runtime dependencies** — Generated parsers are self-contained
- **Semantic analysis** — First/follow sets, left-recursion elimination
- **Structured errors** — Source locations on lexer/parser failures
- **Error recovery & multi-error reporting** — Continues parsing past errors and reports every diagnostic at once
- **Lexer modes** — `mode NAME;` blocks (default + named) with `pushMode` / `popMode` / `mode` commands
- **Visitor / listener** — Rust target emits `<Name>Visitor` / `<Name>Listener` traits plus `walk_*` tree-walkers
- **MCP server** — `pargen-mcp` binary with 5 tools for AI integration
- **Structured logging** — `tracing`-based output with `RUST_LOG` support

## Supported Target Languages

| Language | Status | Output file |
|---|---|---|
| Rust | Supported | `{name}_parser.rs` |
| Go | Supported | `{name}_parser.go` |
| TypeScript | Supported | `{name}_parser.ts` |
| Python | Supported | `{name}_parser.py` |
| Java | Supported | `{name}_parser.java` |
| C | Supported | `{name}_parser.h` |
| C++ | Supported | `{name}_parser.hpp` |
| Tree-sitter | Supported | `{name}_parser.js` |

> The C and C++ targets emit a single header-only file (`*.h` / `*.hpp`) containing both the lexer and the parser. Tree-sitter output is a `grammar.js` module (the file is named `{name}_parser.js`).

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
cargo test          # 175 tests

# The crate ships two binaries (`pargen`, `pargen-mcp`), so `cargo run` needs
# an explicit `--bin pargen` unless a `default-run` is set in Cargo.toml.
RUST_LOG=debug cargo run --bin pargen -- generate -g path/to/grammar.g4 -l rust -o output/
```

## Error Recovery

Both the lexer and parser report multiple diagnostics at once instead of
stopping at the first error. The CLI `check` and `generate` commands print
every diagnostic with its `severity: line:column: message` and any
attached `note`. Internally:

- `G4Lexer::tokenize_recover()` returns `(Vec<G4Token>, Vec<Diagnostic>)`
- `G4Parser::parse_recover()` returns `(Option<Grammar>, Vec<Diagnostic>)`
- `pargen::parse_grammar_source_detailed(source)` ties both together and
  returns `(Option<Grammar>, Vec<Diagnostic>)`. The grammar is `None` only
  if even the header (`grammar X;`) failed to parse.

This makes authoring grammars much faster — fix several errors at once
instead of one per re-run.

## Lexer Modes

`pargen` understands ANTLR4's `mode` syntax for context-sensitive
lexing. Rules declared after a `mode NAME;` header belong to that mode;
the default mode is implicit and contains rules that appear before any
mode declaration or are not inside a mode block.

```antlr
grammar Json;

start: value;

STRING: '"' -> pushMode(STRING) ;
NUMBER: [0-9]+ ;
WS: [ \t\r\n]+ -> skip ;

mode STRING;
CONTENT: ~'"'+ ;
END: '"' -> popMode ;
```

The Rust target emits a `LexerMode` enum, a `mode` field on the lexer,
and per-mode dispatch functions. `pushMode(NAME)`, `popMode`, and
`mode(NAME)` lexer commands translate to assignments that update the
mode state after a token matches. Call `Grammar::lexer_modes()` to list
all modes declared in a grammar (always including `DEFAULT_MODE`).

> **Rust target only.** The other 7 targets currently treat every rule as
> belonging to the default mode — a `mode NAME;` block is parsed and
> recorded, but the generated lexer does not switch modes at runtime.

## Visitors and Listeners (Rust target)

The Rust target emits two tree-walker traits over the generated `<Name>Node`
CST, plus traversal functions:

```rust
pub trait CalculatorVisitor {
    fn visit_expr(&mut self, _node: &CalculatorNode) {}
    fn visit_token(&mut self, _node: &CalculatorNode) {}
}

pub trait CalculatorListener {
    fn enter_expr(&mut self, _node: &CalculatorNode) {}
    fn exit_expr(&mut self, _node: &CalculatorNode) {}
    fn visit_token(&mut self, _node: &CalculatorNode) {}
}
```

- `walk_calculator(visitor, &node)` — visitor DFS; calls `visit_<rule>` *before* recursing.
- `walk_calculator_listen(listener, &node)` — listener DFS; brackets children between `enter_<rule>` and `exit_<rule>`.

All trait methods have no-op defaults, so you override only what you need.
The walkers accept trait objects (`?Sized` bound), so `Box<dyn Visitor>` works.

```rust
struct PrintDots;
impl CalculatorVisitor for PrintDots {
    fn visit_expr(&mut self, n: &CalculatorNode) {
        println!("[expr with {} children]", match n {
            CalculatorNode::expr { children } => children.len(),
            _ => 0,
        });
    }
}
let tree = parser.parse_expr()?;
walk_calculator(&mut PrintDots, &tree);
```

> **Rust target only.** The other 7 targets emit language-idiomatic ASTs
> without a typed CST enum root, so the same machinery does not translate
> cleanly yet.

## License

Apache-2.0
