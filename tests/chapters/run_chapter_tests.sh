#!/bin/bash
# run_chapter_tests.sh — 全章テストの実行ラッパー
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "===== Chapter Snapshot Tests ====="
echo

TOTAL_PASS=0; TOTAL_FAIL=0

for ch in 06 07 08 09 10 11; do
    if bash "$SCRIPT_DIR/test_ch${ch}.sh"; then
        TOTAL_PASS=$((TOTAL_PASS + 1))
    else
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
    fi
    echo
done

echo "===== Chapter Tests Summary: $TOTAL_PASS chapters passed, $TOTAL_FAIL failed ====="
[ "$TOTAL_FAIL" -eq 0 ]
