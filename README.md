# mini-scheme

書籍『Rustで作るSchemeインタプリタ — ソフトウェアサイエンスの真髄を体験する』の実装コードです。

## ファイル構成

| ファイル | 説明 |
|---|---|
| [Cargo.toml](Cargo.toml) | Rustプロジェクト設定 |
| [src/main.rs](src/main.rs) | 完成版インタプリタ（Chapter 5〜10 の全コードを統合） |
| [mini-eval.scm](mini-eval.scm) | Schemeで書かれたSchemeインタプリタ — メタ循環評価器（Chapter 4） |

### 章ごとのコードスナップショット

各章の時点でのソースコードです。章を読み進めながら段階的にビルドできます。

| ディレクトリ | 対応する章 | 内容 |
|---|---|---|
| [chapters/ch04/mini-eval.scm](chapters/ch04/mini-eval.scm) | Chapter 4: メタ循環評価器 | SchemeによるSchemeインタプリタ |
| [chapters/ch05/main.rs](chapters/ch05/main.rs) | Chapter 5: 字句解析 | トークナイザ（Lexer） |
| [chapters/ch06/main.rs](chapters/ch06/main.rs) | Chapter 6: 構文解析 | パーサー（Parser）+ Lexer |
| [chapters/ch07/main.rs](chapters/ch07/main.rs) | Chapter 7: 評価器 | Evaluator + 環境 + 特殊形式 |
| [chapters/ch08/main.rs](chapters/ch08/main.rs) | Chapter 8: 関数とクロージャ | Lambda + レキシカルスコープ |
| [chapters/ch09/main.rs](chapters/ch09/main.rs) | Chapter 9: 組み込み関数 | car/cdr/cons/算術/比較/型述語 |
| [chapters/ch10/main.rs](chapters/ch10/main.rs) | Chapter 10: REPL | 対話的実行環境の完成 |

### その他

| ファイル | 説明 |
|---|---|
| [icon/](icon/) | DrK-Labo アイコン |

## ビルドと実行

```bash
# ビルド
cargo build

# REPL を起動
cargo run
```

```
mini-scheme v0.1.0
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

## メタ循環評価器（Scheme版）の実行

[Gauche](https://practical-scheme.net/gauche/) が必要です。

```bash
gosh mini-eval.scm
```

```scheme
gosh> (my-repl)
mini> (+ 1 2 3)
6
mini> (def (make-adder n) (lambda (x) (+ n x)))
make-adder
mini> ((make-adder 5) 100)
105
```

## テスト

本リポジトリには4種類のテストがあります。

| 種類 | ファイル | テスト数 | 内容 |
|------|--------|--------|------|
| ユニットテスト | `src/main.rs` 末尾 | 82個 | トークナイザ、パーサー、評価器、組み込み関数、エラー処理 |
| 統合テスト | `tests/integration.rs` | 9個 | REPLバイナリの起動・入出力・終了 |
| 章スナップショットテスト | `tests/chapters/test_ch05.sh` 〜 `test_ch10.sh` | 各章数件 | 各章のコードが正しくビルド・実行できるか |
| Schemeテスト | `tests/scheme/test_eval.scm` | 26個 | メタ循環評価器（mini-eval.scm）の動作検証 |

### テストの実行

```bash
# Rust ユニットテスト + 統合テスト
cargo test

# 各章スナップショットのビルド・実行テスト
bash tests/chapters/run_chapter_tests.sh

# メタ循環評価器のテスト（Gauche が必要）
bash tests/scheme/run_scheme_tests.sh

# 上記すべてを一括実行
bash run_tests.sh
```

ユニットテストは `src/main.rs` のファイル末尾に `#[cfg(test)]` で配置しています。これはRustの標準的な慣習で、テストコードは `cargo test` 実行時にだけコンパイルされ、本番バイナリには含まれません。テストを含まない実装コードは `chapters/ch10/main.rs` を参照してください。

テストの書き方の詳細は、書籍の **Appendix D: テストの書き方** を参照してください。

## ライセンス

MIT
