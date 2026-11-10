# mini-scheme v1.0 — Initial Release

The implementation code for the Japanese edition of the book (published 2026-04-12). The `v1.0` tag carries Japanese comments.

## What's included

### The Rust interpreter (`src/main.rs`)

An interpreter for "mini-Scheme", a subset of Scheme. It provides:

- **Lexical analysis**: breaks a string into tokens (numbers, strings, symbols, parentheses, quotes)
- **Parsing**: a recursive-descent parser that builds S-expressions (the Value enum) from a token sequence
- **Evaluator**: evaluates expressions with `eval` / `apply_builtin`. Supports the special forms `if`, `def`, `lambda`, `quote`, `begin`, `set!`, `cond`, `let`, `and`, `or`
- **Environments**: lexical scope through a chain of `HashMap` frames, shared with `Rc<RefCell<Env>>`
- **Closures**: first-class functions that capture the environment of their definition
- **Built-in functions**: arithmetic (`+`, `-`, `*`, `/`), comparison (`=`, `<`, `>`, `<=`, `>=`), list operations (`car`, `cdr`, `cons`, `list`, `append`), type predicates (`null?`, `pair?`, `number?`, `string?`, `symbol?`, `procedure?`, `list?`), equivalence (`eq?`, `equal?`), I/O (`display`, `newline`), and others (`not`, `length`, `map`, `apply`)
- **REPL**: an interactive environment that accepts multi-line input

### The Scheme metacircular evaluator (`mini-eval.scm`)

A Scheme interpreter written in Scheme, running on Gauche. The book explains it in Chapter 4.

### Code snapshots by chapter (`chapters/`)

Source code that runs as of the end of each chapter of the book. It covers each stage: Chapter 4 (the Scheme version) and Chapters 6–11 (the Rust version).

## Tests

Includes 120 unit tests, 9 integration tests, build tests for the chapter snapshots, and behavior tests for the Scheme version (26).

```bash
bash run_tests.sh    # run all the tests at once
```

## Requirements

- **Rust**: Edition 2024 (rustc 1.85 or later)
- **Gauche**: needed to run the metacircular evaluator (optional)

## License

MIT
