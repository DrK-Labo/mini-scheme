#!/bin/bash
# run_tests.sh — runs the full test suite
#
# Usage:
#   bash run_tests.sh
#
# What it runs:
#   1. cargo test (unit tests + integration tests)
#   2. chapter snapshot tests (built directly with rustc)
#   3. Scheme tests (need gosh; skipped if it is missing)
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "========================================"
echo " mini-scheme — Full Test Suite"
echo "========================================"
echo

TOTAL_PASS=0; TOTAL_FAIL=0

# Part 1+2: Rust unit tests + integration tests
echo "--- cargo test ---"
if (cd "$SCRIPT_DIR" && cargo test 2>&1); then
    TOTAL_PASS=$((TOTAL_PASS + 1))
else
    TOTAL_FAIL=$((TOTAL_FAIL + 1))
fi
echo

# Part 3: chapter snapshot tests
if bash "$SCRIPT_DIR/tests/chapters/run_chapter_tests.sh"; then
    TOTAL_PASS=$((TOTAL_PASS + 1))
else
    TOTAL_FAIL=$((TOTAL_FAIL + 1))
fi
echo

# Part 4: Scheme tests
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
