// tests/integration.rs — 完成版バイナリの統合テスト
//
// cargo test --test integration で実行

use std::io::Write;
use std::process::{Command, Stdio};

/// バイナリにstdinを送り、stdoutを返すヘルパー
fn run_repl(input: &str) -> String {
    // cargo test が配置するバイナリのパスを環境変数から取得
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

/// 出力からプロンプトやバナーを除去し、結果行だけ返す
///
/// REPL の出力は "mini> 6" のように prompt + 結果が同一行になる。
/// prompt 部分を剥がして結果だけを取り出す。
fn extract_results(output: &str) -> Vec<String> {
    let mut results = Vec::new();
    for line in output.lines() {
        // バナー行をスキップ
        if line.starts_with("mini-scheme")
            || line.starts_with("Type (exit)")
            || line.is_empty()
        {
            continue;
        }
        // プロンプトを剥がす
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
fn repl_nil_not_printed() {
    // begin の結果が Nil のときは何も出力しない
    let out = run_repl("(begin)\n(+ 1 1)\n(exit)\n");
    let results = extract_results(&out);
    assert_eq!(results, vec!["2"]);
}

#[test]
fn repl_multiline_input() {
    // 複数行にわたる入力（括弧が閉じるまで継続）
    let out = run_repl("(+ 1\n   2\n   3)\n(exit)\n");
    let results = extract_results(&out);
    assert_eq!(results, vec!["6"]);
}

#[test]
fn repl_error_recovery() {
    // エラーが出ても REPL は続行する
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
    // EOF で終了
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
