[日本語](https://github.com/DrK-Labo/mini-scheme/tree/main) | **English**

# mini-scheme

> A small Scheme interpreter for learning, written in about 1,000 lines of Rust

This is the companion code for the book *Crafting a Scheme Interpreter in Rust*. It builds a Scheme interpreter step by step — lexical analysis, parsing, the evaluator, closures, lexical scope, and the REPL. The code stays small, but it is expressive enough to run the Y combinator, and the repository includes the metacircular evaluator from Chapter 4 (run on Gauche). Tail-call optimization is left as an exercise (Chapter 13).

This `en` branch carries the same code as `main`, with English comments. `main` carries Japanese comments for the Japanese edition.

## Try it

```bash
git clone https://github.com/DrK-Labo/mini-scheme.git
cd mini-scheme
git checkout v1.1-en
cargo run
```

```
mini-scheme v1.1.0
Type (exit) to quit.

mini> (+ 1 2 3)
6
mini> (def (factorial n) (if (= n 0) 1 (* n (factorial (- n 1)))))
factorial
mini> (factorial 10)
3628800
mini> (exit)
Bye!
```

## What it can run

### Factorial (recursion)

```scheme
(def (factorial n)
  (if (= n 0)
      1
      (* n (factorial (- n 1)))))

(factorial 10)
; => 3628800
```

### Closures (captured environments and lexical scope)

```scheme
(def (make-adder n)
  (lambda (x) (+ n x)))

(def add-5 (make-adder 5))
(add-5 100)
; => 105
```

### The Y combinator (recursion without names)

```scheme
(def Y (lambda (F)
         ((lambda (x) (F (lambda (v) ((x x) v))))
          (lambda (x) (F (lambda (v) ((x x) v)))))))

(def F (lambda (f)
         (lambda (n)
           (if (= n 0) 1 (* n (f (- n 1)))))))

(def factorial (Y F))
(factorial 10)
; => 3628800
```

This builds a recursive function without using any name on the right-hand side of `def`: the Y combinator of the lambda calculus runs as is in mini-scheme. For more, see the [Zenn article on the name "Y combinator"](https://zenn.dev/drk_laboratory) (in Japanese).

## About the book

This repository is the companion code for the book **_Crafting a Scheme Interpreter in Rust — Hands-on Implementation of Lexers, Parsers, Evaluators, Closures & Lexical Scope_**. About 1,000 lines of Rust are assembled chapter by chapter, in the order lexical analysis → parsing → evaluator → closures → REPL.

| | |
|---|---|
| English edition | [Amazon Kindle](https://www.amazon.com/dp/B0HS51CVQG) — code: the `v1.1-en` tag on this `en` branch |
| Japanese edition | [Amazon Kindle (Japan)](https://www.amazon.co.jp/dp/B0GX2VM5W5) — code: the `main` branch (`v1.0` tag for the first edition) |
| Author | Dr.K Laboratory |
| X | [@DrKLaboratory](https://x.com/DrKLaboratory) |
| Zenn | [drk_laboratory](https://zenn.dev/drk_laboratory) |

## Following along chapter by chapter

If you want to work through the book hands-on, [`chapters/`](chapters/) holds the code as it runs at the end of each chapter. You can build each chapter on its own and see what works up to that point.

```bash
# Example: run the state at Chapter 8 (up to the evaluator)
rustc chapters/ch08/main.rs -o ch08-mini-scheme
./ch08-mini-scheme
```

Even without the book, reading `src/main.rs` (the finished version, about 1,000 lines) gives you the whole picture of the interpreter. In a single file you can follow the typical patterns of a language implementation: an evaluator written with Rust's `enum` and `match`, an environment chain built on `Rc<RefCell<T>>`, and the mutual recursion of `eval` and `apply`.

## Files

| File | Description |
|---|---|
| [Cargo.toml](Cargo.toml) | Rust project settings |
| [src/main.rs](src/main.rs) | The finished interpreter (all the code from Chapters 6–11 combined) |
| [mini-eval.scm](mini-eval.scm) | A Scheme interpreter written in Scheme — the metacircular evaluator (Chapter 4) |

### Code snapshots by chapter

| Directory | Chapter | Contents |
|---|---|---|
| [chapters/ch04/mini-eval.scm](chapters/ch04/mini-eval.scm) | Chapter 4: The Metacircular Evaluator | A Scheme interpreter in Scheme |
| [chapters/ch06/main.rs](chapters/ch06/main.rs) | Chapter 6: Lexical Analysis | Tokenizer (lexer) |
| [chapters/ch07/main.rs](chapters/ch07/main.rs) | Chapter 7: Parsing | Parser + lexer |
| [chapters/ch08/main.rs](chapters/ch08/main.rs) | Chapter 8: The Evaluator | Evaluator + environment + special forms |
| [chapters/ch09/main.rs](chapters/ch09/main.rs) | Chapter 9: Functions and Closures | Lambda + lexical scope |
| [chapters/ch10/main.rs](chapters/ch10/main.rs) | Chapter 10: Built-In Functions | car/cdr/cons, arithmetic, comparison, type predicates |
| [chapters/ch11/main.rs](chapters/ch11/main.rs) | Chapter 11: The REPL | The finished interactive environment |

### Other

| File | Description |
|---|---|
| [icon/](icon/) | DrK-Labo icon |

## Build and run

```bash
# Build
cargo build

# Start the REPL
cargo run
```

## Running the metacircular evaluator (Scheme version)

To run the Scheme interpreter written in Scheme, you need [Gauche](https://practical-scheme.net/gauche/).

```bash
gosh -l ./mini-eval.scm
```

```
gosh> (my-repl)
mini> (+ 1 2 3)
6
mini> (def (make-adder n) (lambda (x) (+ n x)))
make-adder
mini> ((make-adder 5) 100)
105
```

## Tests

This repository has four kinds of tests.

| Kind | File | Count | What it covers |
|------|--------|--------|------|
| Unit tests | end of `src/main.rs` | 125 | Tokenizer, parser, evaluator, built-in functions, error handling |
| Integration tests | `tests/integration.rs` | 10 | Starting the REPL binary, input/output, and exit |
| Chapter snapshot tests | `tests/chapters/test_ch06.sh` – `test_ch11.sh` | a few per chapter | Whether each chapter's code builds and runs correctly |
| Scheme tests | `tests/scheme/test_eval.scm` | 25 | Behavior of the metacircular evaluator (mini-eval.scm) |

### Running the tests

```bash
# Rust unit tests + integration tests
cargo test

# Build-and-run tests for each chapter snapshot
bash tests/chapters/run_chapter_tests.sh

# Tests for the metacircular evaluator (needs Gauche)
bash tests/scheme/run_scheme_tests.sh

# Run all of the above at once
bash run_tests.sh
```

The unit tests sit at the end of `src/main.rs`, inside `#[cfg(test)]`. That is the standard Rust convention: test code is compiled only when `cargo test` runs and never ends up in the production binary. For the implementation alone, without tests, see `chapters/ch11/main.rs`.

For how to write the tests, see **Appendix D: Using the GitHub Repository and Writing Tests** in the book.

## License

MIT
