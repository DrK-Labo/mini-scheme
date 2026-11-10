# Changelog

Change history of the source code repository for the book *Crafting a Scheme Interpreter in Rust*
(the Japanese edition was published first).
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [v1.1-en]

The English-comment build that accompanies the English edition (version 1.1).
The code is the same as on `main`; only the comments and documents differ.

### Changed
- All comments in the code and tests, `README.md`, and this changelog are in English
- The version is now 1.1.0 (the REPL banner shows `mini-scheme v1.1.0`)

### Fixed
- Evaluating `(def)` made the interpreter panic. It read `elems[1]` before checking the
  length; it now returns an `Err` (`eval_define`)
- `(let () ...)` did not open a new scope: when the binding list was empty, an inner `def`
  leaked out to the enclosing scope (`eval_let`)
- Unary `(/ 0)` returned `inf` instead of a division-by-zero error. The check looked only at
  the second and later arguments (`"/"` in `apply_builtin`)

- The REPL did not print `Value::Nil`. The suppression meant to hide the return value of
  `display` **also swallowed legitimate empty lists**, such as the result of `(cdr '(1))` or `(list)`.
  Gauche returns `()` for the same input
- The lexer looped forever on Unicode whitespace such as the ideographic space or NBSP.
  Whitespace is now detected with `c.is_whitespace()` (`tokenize`).
  Added a regression test (unit tests 124 → 125)

### Added
- Regression tests for the three fixes above (unit tests 120 → 124)
- Renamed `repl_nil_not_printed` to `repl_nil_is_printed`, and added a regression test for
  printing the empty list (integration tests 9 → 10)

## [v1.0] - 2026-04-12

Initial release, with Japanese comments, for the Japanese edition.

### Added
- The finished interpreter (`src/main.rs`) — with 120 unit tests and 9 integration tests
- The Scheme metacircular evaluator (`mini-eval.scm`)
- Snapshots for each chapter (`chapters/ch04`–`ch11`) — 37 snapshot tests
- Scheme tests (`tests/scheme/`) — 50
- MIT LICENSE
