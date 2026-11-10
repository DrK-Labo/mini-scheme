#!/bin/bash
# test_ch11.sh — tests for Chapter 11: The REPL
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
SRC="$PROJ_DIR/chapters/ch11/main.rs"
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

echo "=== Chapter 11: REPL ==="
rustc "$SRC" -o "$TMPDIR/ch11" 2>/dev/null

# Test 1: basic arithmetic
OUTPUT=$(echo '(+ 1 2 3)
(exit)' | "$TMPDIR/ch11")
check "arithmetic"  "6"

# Test 2: def + call
OUTPUT=$(echo '(def (square x) (* x x))
(square 7)
(exit)' | "$TMPDIR/ch11")
check "def and call"  "49"

# Test 3: multi-line input
OUTPUT=$(echo '(+ 1
   2
   3)
(exit)' | "$TMPDIR/ch11")
check "multiline"  "6"

# Test 4: Bye! on exit
OUTPUT=$(echo '(exit)' | "$TMPDIR/ch11")
check "exit message"  "Bye!"

# Test 5: Bye! on EOF
OUTPUT=$(echo -n "" | "$TMPDIR/ch11")
check "eof message"  "Bye!"

# Test 6: factorial
OUTPUT=$(echo '(def (factorial n) (if (= n 0) 1 (* n (factorial (- n 1)))))
(factorial 10)
(exit)' | "$TMPDIR/ch11")
check "factorial"  "3628800"

echo "  $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
