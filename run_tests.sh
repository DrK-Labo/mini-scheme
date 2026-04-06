#!/bin/bash
# run_tests.sh — 全テスト統合スクリプト
#
# 使い方:
#   bash run_tests.sh
#
# 実行内容:
#   1. cargo test（ユニットテスト + 統合テスト）
#   2. 各章スナップショットテスト（rustc で直接ビルド）
#   3. Scheme テスト（gosh が必要、なければスキップ）
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "========================================"
echo " mini-scheme — Full Test Suite"
echo "========================================"
echo

TOTAL_PASS=0; TOTAL_FAIL=0

# Part 1+2: Rust ユニットテスト + 統合テスト
echo "--- cargo test ---"
if (cd "$SCRIPT_DIR" && cargo test 2>&1); then
    TOTAL_PASS=$((TOTAL_PASS + 1))
else
    TOTAL_FAIL=$((TOTAL_FAIL + 1))
fi
echo

# Part 3: 各章スナップショットテスト
if bash "$SCRIPT_DIR/tests/chapters/run_chapter_tests.sh"; then
    TOTAL_PASS=$((TOTAL_PASS + 1))
else
    TOTAL_FAIL=$((TOTAL_FAIL + 1))
fi
echo

# Part 4: Scheme テスト
if bash "$SCRIPT_DIR/tests/scheme/run_scheme_tests.sh"; then
    TOTAL_PASS=$((TOTAL_PASS + 1))
else
    TOTAL_FAIL=$((TOTAL_FAIL + 1))
fi
echo

echo "========================================"
echo " Results: $TOTAL_PASS suites passed, $TOTAL_FAIL failed"
echo "========================================"
[ "$TOTAL_FAIL" -eq 0 ]
