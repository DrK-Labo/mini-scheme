#!/bin/bash
# run_scheme_tests.sh — runs the Scheme tests
#
# Skips (exit 0) if Gauche (gosh) is not installed.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

if ! command -v gosh &>/dev/null; then
    echo "===== Scheme Tests: SKIPPED (gosh not found) ====="
    exit 0
fi

echo "===== Scheme Tests ====="
echo

TOTAL_PASS=0; TOTAL_FAIL=0

# Test 1: mini-eval.scm (root)
echo "--- mini-eval.scm (root) ---"
if gosh -l "$PROJ_DIR/mini-eval.scm" "$SCRIPT_DIR/test_eval.scm"; then
    TOTAL_PASS=$((TOTAL_PASS + 1))
else
    TOTAL_FAIL=$((TOTAL_FAIL + 1))
fi
echo

# Test 2: chapters/ch04/mini-eval.scm
echo "--- chapters/ch04/mini-eval.scm ---"
if gosh -l "$PROJ_DIR/chapters/ch04/mini-eval.scm" "$SCRIPT_DIR/test_eval.scm"; then
    TOTAL_PASS=$((TOTAL_PASS + 1))
else
    TOTAL_FAIL=$((TOTAL_FAIL + 1))
fi
echo

echo "===== Scheme Tests Summary: $TOTAL_PASS passed, $TOTAL_FAIL failed ====="
[ "$TOTAL_FAIL" -eq 0 ]
