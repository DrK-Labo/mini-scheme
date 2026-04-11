#!/bin/bash
# test_ch08.sh — Chapter 8: 評価器 のテスト
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
SRC="$PROJ_DIR/chapters/ch08/main.rs"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

PASS=0; FAIL=0

check() {
    local desc="$1" pattern="$2"
    if echo "$OUTPUT" | grep -qF "$pattern"; then
        PASS=$((PASS + 1))
    else
        echo "  FAIL: $desc (expected: $pattern)"
        FAIL=$((FAIL + 1))
    fi
}

echo "=== Chapter 8: Evaluator ==="
rustc "$SRC" -o "$TMPDIR/ch08" 2>/dev/null
OUTPUT=$("$TMPDIR/ch08")

check "addition"          '(+ 1 2 3) => 6'
check "nested expr"       '(* 2 (+ 3 4)) => 14'
check "def variable"      'x => 42'
check "def function"      '(square 7) => 49'
check "if expression"     '(if #t "yes" "no") => "yes"'
check "factorial"         '(factorial 10) => 3628800'

echo "  $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
