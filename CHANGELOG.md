# Changelog

本書『Rustで作るSchemeインタプリタ』のソースコードリポジトリの変更履歴です。
形式は [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) に基づきます。

## [Unreleased]

### Fixed
- `(def)` を評価するとインタプリタがパニックしていた。長さ検査より先に `elems[1]` を
  参照していたのが原因で、`Err` を返すように修正（`eval_define`）
- `(let () ...)` が新しいスコープを作らず、束縛リストが空のときだけ内側の `def` が
  外側へ漏れていた（`eval_let`）
- 単項の `(/ 0)` がゼロ除算エラーにならず `inf` を返していた。検査が第2引数以降しか
  見ていなかった（`apply_builtin` の `"/"`）

### Changed
- 上記3件の回帰テストを追加。ユニットテスト 120個 → 124個

## [v1.0] - 2026-04-12

初版リリース。

### Added
- 完成版インタプリタ（`src/main.rs`）— ユニットテスト120個、統合テスト9個を含む
- Schemeメタ循環評価器（`mini-eval.scm`）
- 各章スナップショット（`chapters/ch04`〜`ch11`）— スナップショットテスト37個
- Schemeテスト（`tests/scheme/`）— 50個
- MIT LICENSE
