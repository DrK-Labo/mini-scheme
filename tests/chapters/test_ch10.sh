#!/bin/bash
# test_ch10.sh — Chapter 10: REPL のテスト
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

echo "=== Chapter 10: REPL ==="
rustc "$SRC" -o "$TMPDIR/ch10" 2>/dev/null

# テスト1: 基本的な算術
OUTPUT=$(echo '(+ 1 2 3)
(exit)' | "$TMPDIR/ch10")
check "arithmetic"  "6"

# テスト2: def + 呼び出し
OUTPUT=$(echo '(def (square x) (* x x))
(square 7)
(exit)' | "$TMPDIR/ch10")
check "def and call"  "49"

# テスト3: 複数行入力
OUTPUT=$(echo '(+ 1
   2
   3)
(exit)' | "$TMPDIR/ch10")
check "multiline"  "6"

# テスト4: exit で Bye!
OUTPUT=$(echo '(exit)' | "$TMPDIR/ch10")
check "exit message"  "Bye!"

# テスト5: EOF で Bye!
OUTPUT=$(echo -n "" | "$TMPDIR/ch10")
check "eof message"  "Bye!"

# テスト6: factorial
OUTPUT=$(echo '(def (factorial n) (if (= n 0) 1 (* n (factorial (- n 1)))))
(factorial 10)
(exit)' | "$TMPDIR/ch10")
check "factorial"  "3628800"

echo "  $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
