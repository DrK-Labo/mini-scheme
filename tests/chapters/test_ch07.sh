#!/bin/bash
# test_ch07.sh — Chapter 7: 構文解析 のテスト
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
SRC="$PROJ_DIR/chapters/ch07/main.rs"
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

echo "=== Chapter 7: Parser ==="
rustc "$SRC" -o "$TMPDIR/ch07" 2>/dev/null
OUTPUT=$("$TMPDIR/ch07")

check "simple list"        'Parsed: (+ 1 2)'
check "nested def"         'Parsed: (def (square x) (* x x))'
check "quote sugar"        'Parsed: (quote (1 2 3))'
check "nested expr"        'Parsed: (+ 1 (* 2 3))'
check "empty list → Nil"   'Parsed: ()'

echo "  $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
