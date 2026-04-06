;;;; test_eval.scm — mini-eval.scm のテストスイート
;;;;
;;;; 使い方:
;;;;   gosh -l mini-eval.scm tests/scheme/test_eval.scm
;;;;
;;;; mini-eval.scm を load した状態で実行し、my-eval を直接テストする。

(define *test-pass* 0)
(define *test-fail* 0)

(define (test-equal desc expected actual)
  (if (equal? expected actual)
      (begin
        (set! *test-pass* (+ *test-pass* 1)))
      (begin
        (set! *test-fail* (+ *test-fail* 1))
        (display "  FAIL: ")
        (display desc)
        (display " expected=")
        (display expected)
        (display " actual=")
        (display actual)
        (newline))))

(define (test-error desc thunk)
  (guard (e (#t (set! *test-pass* (+ *test-pass* 1))))
    (thunk)
    ;; thunk が正常終了した場合 → 期待に反してエラーなし
    (set! *test-fail* (+ *test-fail* 1))
    (display "  FAIL: ")
    (display desc)
    (display " (expected error, but succeeded)")
    (newline)))

;;; グローバル環境を初期化
(set! *global-env* (make-global-env))

(display "=== mini-eval.scm Tests ===")
(newline)

;; --- 自己評価 ---
(test-equal "number"   42     (my-eval 42 '()))
(test-equal "string"   "hi"   (my-eval "hi" '()))
(test-equal "bool #t"  #t     (my-eval #t '()))
(test-equal "bool #f"  #f     (my-eval #f '()))

;; --- quote ---
(test-equal "quote"    '(1 2) (my-eval '(quote (1 2)) '()))

;; --- 算術 ---
(test-equal "add"      6      (my-eval '(+ 1 2 3) '()))
(test-equal "sub"      7      (my-eval '(- 10 3) '()))
(test-equal "mul"      24     (my-eval '(* 2 3 4) '()))
(test-equal "div"      5      (my-eval '(/ 10 2) '()))

;; --- 比較 ---
(test-equal "eq true"  #t     (my-eval '(= 3 3) '()))
(test-equal "eq false" #f     (my-eval '(= 3 4) '()))
(test-equal "lt"       #t     (my-eval '(< 1 2) '()))

;; --- if ---
(test-equal "if true"  1      (my-eval '(if #t 1 2) '()))
(test-equal "if false" 2      (my-eval '(if #f 1 2) '()))

;; --- def (variable) ---
(set! *global-env* (make-global-env))
(test-equal "def var returns name" 'x (my-eval '(def x 42) '()))
(test-equal "def var lookup"       42  (my-eval 'x '()))

;; --- def (function) ---
(set! *global-env* (make-global-env))
(my-eval '(def (square n) (* n n)) '())
(test-equal "def func call" 25 (my-eval '(square 5) '()))

;; --- lambda ---
(test-equal "lambda call" 11
  (my-eval '((lambda (x) (+ x 1)) 10) '()))

;; --- begin ---
(test-equal "begin" 3
  (my-eval '(begin 1 2 3) '()))

;; --- クロージャ ---
(set! *global-env* (make-global-env))
(my-eval '(def (make-adder n) (lambda (x) (+ n x))) '())
(my-eval '(def add5 (make-adder 5)) '())
(test-equal "closure" 15 (my-eval '(add5 10) '()))

;; --- 再帰 ---
(set! *global-env* (make-global-env))
(my-eval '(def (factorial n) (if (= n 0) 1 (* n (factorial (- n 1))))) '())
(test-equal "factorial" 3628800 (my-eval '(factorial 10) '()))

;; --- リスト操作 ---
(test-equal "car"  1      (my-eval '(car '(1 2 3)) '()))
(test-equal "cdr"  '(2 3) (my-eval '(cdr '(1 2 3)) '()))
(test-equal "cons" '(0 1) (my-eval '(cons 0 '(1)) '()))

;; --- エラー ---
(test-error "undefined var" (lambda () (my-eval 'xyz '())))

;; --- 結果表示 ---
(newline)
(display "  ")
(display *test-pass*)
(display " passed, ")
(display *test-fail*)
(display " failed")
(newline)

(when (> *test-fail* 0)
  (exit 1))
