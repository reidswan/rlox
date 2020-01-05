# RLox

A [Rust](https://www.rust-lang.org/) implementation of the Lox programming language featured in [Crafting Interpreters](http://craftinginterpreters.com).

WIP: only a partial version of the tree walk interpreter is currently available.

## Differences from the spec:
- the ternary `<expr> ? <expr> : <expr>` expression form
- escapes strings in the scanner: `print "He said, \"Go home.\"\nShe did.";` will print 
```
He said, "Go home."
She did.
```
- no empty `var` declarations; `var x;` is a syntax error.
- REPL has directives prepended by a `.`: `.exit` and `.help`

## Build

Uses Rust's Cargo build tool.

Debug build: 
```bash
cargo build
```

Release build:
```bash
cargo build --release
```

## Run

REPL mode: 
```bash
cargo run --release
```

Script mode:
```bash
cargo run --release /path/to/script.lox
```

## Examples

Example Lox scripts are located in `examples/`
