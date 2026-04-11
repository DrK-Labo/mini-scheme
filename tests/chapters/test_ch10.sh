#!/bin/bash
# test_ch10.sh — Chapter 10: 組み込み関数 のテスト
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
SRC="$PROJ_DIR/chapters/ch10/main.rs"
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

echo "=== Chapter 10: Builtins ==="
rustc "$SRC" -o "$TMPDIR/ch10" 2>/dev/null
OUTPUT=$("$TMPDIR/ch10")

check "car"              "(car '(1 2 3)) => 1"
check "cdr"              "(cdr '(1 2 3)) => (2 3)"
check "cons"             "(cons 0 '(1 2 3)) => (0 1 2 3)"
check "null?"            "(null? '()) => #t"
check "number?"          "(number? 42) => #t"
check "string?"          '(string? "hello") => #t'
check "user map"         '(my-map (lambda (x) (* x x))'
check "map result"       '=> (1 4 9 16 25)'
check "user filter"      '=> (4 5)'

echo "  $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
