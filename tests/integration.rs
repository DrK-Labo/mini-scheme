// tests/integration.rs

use std::io::Write;
use std::process::{Command, Stdio};

/// Helper that sends stdin to the binary and returns stdout
fn run_repl(input: &str) -> String {
    // cargo test supplies the binary's path in this environment variable
    let bin = env!("CARGO_BIN_EXE_mini-scheme");
    let mut child = Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start mini-scheme");

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).unwrap();
    }

    let output = child.wait_with_output().expect("Failed to wait");
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Strip the prompts and banner from the output, returning only the result lines
fn extract_results(output: &str) -> Vec<String> {
    let mut results = Vec::new();
    for line in output.lines() {
        // skip the banner lines
        if line.starts_with("mini-scheme")
            || line.starts_with("Type (exit)")
            || line.is_empty()
        {
            continue;
        }
        // strip the prompts
        let stripped = line
            .replace("mini> ", "")
            .replace("...   ", "");
        let stripped = stripped.trim();
        if stripped.is_empty() || stripped == "Bye!" {
            continue;
        }
        results.push(stripped.to_string());
    }
    results
}

#[test]
fn repl_arithmetic() {
    let out = run_repl("(+ 1 2 3)\n(exit)\n");
    let results = extract_results(&out);
    assert_eq!(results, vec!["6"]);
}

#[test]
fn repl_def_and_call() {
    let out = run_repl("(def (square x) (* x x))\n(square 7)\n(exit)\n");
    let results = extract_results(&out);
    assert_eq!(results, vec!["square", "49"]);
}

#[test]
fn repl_nil_is_printed() {
    // Nil is printed as the empty list, so that a legitimate result such as
    // (cdr '(1)) is not swallowed (since 2026-08-22; before that it was hidden)
    let out = run_repl("(begin)\n(+ 1 1)\n(exit)\n");
    let results = extract_results(&out);
    assert_eq!(results, vec!["()", "2"]);
}

#[test]
fn repl_prints_empty_list_results() {
    // the result of an expression that returns the empty list must be visible
    let out = run_repl("(cdr '(1))\n(list)\n(exit)\n");
    let results = extract_results(&out);
    assert_eq!(results, vec!["()", "()"]);
}

#[test]
fn repl_multiline_input() {
    // input spanning several lines (continues until the parens close)
    let out = run_repl("(+ 1\n   2\n   3)\n(exit)\n");
    let results = extract_results(&out);
    assert_eq!(results, vec!["6"]);
}

#[test]
fn repl_error_recovery() {
    // the REPL keeps going after an error
    let out = run_repl("(/ 1 0)\n(+ 1 1)\n(exit)\n");
    let results = extract_results(&out);
    assert!(results[0].contains("Error"));
    assert_eq!(results[1], "2");
}

#[test]
fn repl_exit() {
    let out = run_repl("(exit)\n");
    assert!(out.contains("Bye!"));
}

#[test]
fn repl_eof() {
    // exit on EOF
    let out = run_repl("");
    assert!(out.contains("Bye!"));
}

#[test]
fn repl_factorial() {
    let out = run_repl(
        "(def (factorial n) (if (= n 0) 1 (* n (factorial (- n 1)))))\n\
         (factorial 10)\n\
         (exit)\n",
    );
    let results = extract_results(&out);
    assert_eq!(results, vec!["factorial", "3628800"]);
}

#[test]
fn repl_closure() {
    let out = run_repl(
        "(def (make-adder n) (lambda (x) (+ n x)))\n\
         (def add5 (make-adder 5))\n\
         (add5 100)\n\
         (exit)\n",
    );
    let results = extract_results(&out);
    assert_eq!(results, vec!["make-adder", "add5", "105"]);
}
