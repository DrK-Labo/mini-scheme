# mini-scheme

> 約1,000行の Rust で書かれた、学習用の小さな Scheme インタプリタ

字句解析、構文解析、評価器、クロージャ、レキシカルスコープ、REPL までを段階的に実装した、書籍『Rustで作るSchemeインタプリタ』の付録実装コードです。コード規模は抑えつつ、メタ循環評価器・Y コンビネータ・末尾呼び出し最適化といった言語処理系の核心トピックを動かして確かめられる程度の表現力を持たせています。

## 動かしてみる

```bash
git clone https://github.com/DrK-Labo/mini-scheme.git
cd mini-scheme
cargo run
```

```
mini-scheme v1.0.0
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

## 何が動くか

### 階乗（再帰）

```scheme
(def (factorial n)
  (if (= n 0)
      1
      (* n (factorial (- n 1)))))

(factorial 10)
; => 3628800
```

### クロージャ（環境捕捉とレキシカルスコープ）

```scheme
(def (make-adder n)
  (lambda (x) (+ n x)))

(def add-5 (make-adder 5))
(add-5 100)
; => 105
```

### Y コンビネータ（名前なし再帰）

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

`def` の右辺で名前を一切使わずに、再帰関数を構築しています。λ計算の Y コンビネータが mini-scheme でそのまま動く、という実例。詳しくは Zenn 記事「[Y Combinator という名前の話](https://zenn.dev/drk_laboratory)」を参照。

## 書籍について

本リポジトリは、技術書 **『Rustで作るSchemeインタプリタ — Scheme言語の実装で学ぶ、Rustとソフトウェアサイエンス』** の付録実装コードです。約1,000行の Rust コードを、字句解析→構文解析→評価器→クロージャ→REPL の順に章ごとに組み上げる構成。

| | |
|---|---|
| 書籍 | [Amazon Kindle で見る](https://www.amazon.co.jp/dp/B0GX2VM5W5) |
| 配信 | Kindle Unlimited 対象 |
| 著者 | Dr.K Laboratory |
| X | [@DrKLaboratory](https://x.com/DrKLaboratory) |
| Zenn | [drk_laboratory](https://zenn.dev/drk_laboratory) |

## 章を追って読みたい方へ

書籍を読みながら手を動かす場合、各章の終わり時点で動くコードが [`chapters/`](chapters/) に保存されています。章ごとにビルドして、その章までで何が動くかを確かめながら進める設計です。

```bash
# 例：Chapter 8（評価器まで）の状態で動かす
rustc chapters/ch08/main.rs -o ch08-mini-scheme
./ch08-mini-scheme
```

書籍を持っていない方も、`src/main.rs`（完成版、約1,000行）を読むだけでインタプリタの全景が見えます。Rust の `enum` と `match` で書かれた評価器の構造、`Rc<RefCell<T>>` を使った環境チェーンの実装、`eval` と `apply` の相互再帰など、言語処理系の典型的なパターンを単一ファイルで追えます。

## ファイル構成

| ファイル | 説明 |
|---|---|
| [Cargo.toml](Cargo.toml) | Rust プロジェクト設定 |
| [src/main.rs](src/main.rs) | 完成版インタプリタ（Chapter 6〜11 の全コードを統合） |
| [mini-eval.scm](mini-eval.scm) | Scheme で書かれた Scheme インタプリタ — メタ循環評価器（Chapter 4） |

### 章ごとのコードスナップショット

| ディレクトリ | 対応する章 | 内容 |
|---|---|---|
| [chapters/ch04/mini-eval.scm](chapters/ch04/mini-eval.scm) | Chapter 4: メタ循環評価器 | Scheme による Scheme インタプリタ |
| [chapters/ch06/main.rs](chapters/ch06/main.rs) | Chapter 6: 字句解析 | トークナイザ（Lexer） |
| [chapters/ch07/main.rs](chapters/ch07/main.rs) | Chapter 7: 構文解析 | パーサー（Parser）+ Lexer |
| [chapters/ch08/main.rs](chapters/ch08/main.rs) | Chapter 8: 評価器 | Evaluator + 環境 + 特殊形式 |
| [chapters/ch09/main.rs](chapters/ch09/main.rs) | Chapter 9: 関数とクロージャ | Lambda + レキシカルスコープ |
| [chapters/ch10/main.rs](chapters/ch10/main.rs) | Chapter 10: 組み込み関数 | car/cdr/cons/算術/比較/型述語 |
| [chapters/ch11/main.rs](chapters/ch11/main.rs) | Chapter 11: REPL | 対話的実行環境の完成 |

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

## メタ循環評価器（Scheme版）の実行

Scheme で書かれた Scheme インタプリタを動かすには、[Gauche](https://practical-scheme.net/gauche/) が必要です。

```bash
gosh -l mini-eval.scm
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

## テスト

本リポジトリには4種類のテストがあります。

| 種類 | ファイル | テスト数 | 内容 |
|------|--------|--------|------|
| ユニットテスト | `src/main.rs` 末尾 | 120個 | トークナイザ、パーサー、評価器、組み込み関数、エラー処理 |
| 統合テスト | `tests/integration.rs` | 9個 | REPL バイナリの起動・入出力・終了 |
| 章スナップショットテスト | `tests/chapters/test_ch06.sh` 〜 `test_ch11.sh` | 各章数件 | 各章のコードが正しくビルド・実行できるか |
| Scheme テスト | `tests/scheme/test_eval.scm` | 26個 | メタ循環評価器（mini-eval.scm）の動作検証 |

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

ユニットテストは `src/main.rs` のファイル末尾に `#[cfg(test)]` で配置しています。これは Rust の標準的な慣習で、テストコードは `cargo test` 実行時にだけコンパイルされ、本番バイナリには含まれません。テストを含まない実装コードは `chapters/ch11/main.rs` を参照してください。

テストの書き方の詳細は、書籍の **Appendix D: GitHubリポジトリの利用とテストの書き方** を参照してください。

## ライセンス

MIT
