#!/bin/bash
# test_ch08.sh — Chapter 8: 関数とクロージャ のテスト
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

echo "=== Chapter 8: Lambda & Closure ==="
rustc "$SRC" -o "$TMPDIR/ch08" 2>/dev/null
OUTPUT=$("$TMPDIR/ch08")

check "make-adder def"     'make-adder'
check "closure call"       '(add5 3) => 8'
check "counter 1"          '(c) => 1'
check "counter 2"          '(c) => 2'
check "counter 3"          '(c) => 3'

echo "  $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
