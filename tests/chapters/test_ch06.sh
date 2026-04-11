#!/bin/bash
# test_ch06.sh — Chapter 6: 字句解析 のテスト
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
SRC="$PROJ_DIR/chapters/ch06/main.rs"
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

echo "=== Chapter 6: Lexer ==="
rustc "$SRC" -o "$TMPDIR/ch06" 2>/dev/null
OUTPUT=$("$TMPDIR/ch06")

check "tokenize (+ 1 2)"          'Symbol("+")'
check "number token"               'Number(1.0)'
check "string token"               'Str("yes")'
check "bool token"                 'Bool(true)'
check "quote token"                'Quote'
check "paren tokens"               'LParen'

echo "  $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
