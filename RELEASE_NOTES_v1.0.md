# mini-scheme v1.0 — Initial Release

書籍『Rustで作るSchemeインタプリタ — Scheme言語の実装で学ぶ、Rustとソフトウェアサイエンス』の実装コードです。

## 含まれるもの

### Rust版インタプリタ（`src/main.rs`）

Scheme のサブセット「mini-Scheme」のインタプリタです。以下の機能を備えています。

- **字句解析**: 文字列をトークンに分解（数値・文字列・シンボル・括弧・クォート）
- **構文解析**: トークン列からS式（Value enum）を構築する再帰下降パーサー
- **評価器**: `eval` / `apply_builtin` による式の評価。特殊形式（`if`, `def`, `lambda`, `quote`, `begin`, `set!`, `cond`, `let`, `and`, `or`）をサポート
- **環境**: `HashMap` フレーム連鎖によるレキシカルスコープ。`Rc<RefCell<Env>>` で共有
- **クロージャ**: 定義時の環境を捕捉する第一級関数
- **組み込み関数**: 算術（`+`, `-`, `*`, `/`）、比較（`=`, `<`, `>`, `<=`, `>=`）、リスト操作（`car`, `cdr`, `cons`, `list`, `append`）、型述語（`null?`, `pair?`, `number?`, `string?`, `symbol?`, `procedure?`, `list?`）、等価判定（`eq?`, `equal?`）、入出力（`display`, `newline`）、その他（`not`, `length`, `map`, `apply`）
- **REPL**: 複数行入力対応の対話的実行環境

### Schemeメタ循環評価器（`mini-eval.scm`）

Gauche上で動作する、Schemeで書かれたSchemeインタプリタです。書籍 Chapter 4 で解説しています。

### 章ごとのコードスナップショット（`chapters/`）

書籍の各章末時点で動作するソースコードです。Chapter 4（Scheme版）、Chapter 6〜11（Rust版）の各段階を収録しています。

## テスト

120個のユニットテスト、9個の統合テスト、章スナップショットのビルドテスト、Scheme版の動作テスト（26個）を含みます。

```bash
bash run_tests.sh    # 全テスト一括実行
```

## 動作環境

- **Rust**: Edition 2024（rustc 1.85 以降）
- **Gauche**: メタ循環評価器の実行に必要（オプション）

## ライセンス

MIT
