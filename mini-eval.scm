;;;; mini-eval.scm — SchemeでSchemeを書く（メタ循環評価器）
;;;;
;;;; Chapter 4 の完成コード。Gauche (gosh) で実行できます。
;;;;
;;;; 使い方:
;;;;   $ gosh mini-eval.scm
;;;;   gosh> (my-repl)
;;;;   mini> (+ 1 2 3)
;;;;   6

;;; グローバル環境（defineで定義された変数はここに蓄積される）
(define *global-env* '())

;;; ===== 評価器 =====

(define (my-eval expr env)
  (cond
    ;; 数値・文字列・真偽値はそのまま返す（自己評価的）
    ((number? expr) expr)
    ((string? expr) expr)
    ((boolean? expr) expr)

    ;; シンボルは環境から値を探す
    ((symbol? expr)
     (lookup-env expr env))

    ;; リスト（特殊形式 or 関数呼び出し）
    ((pair? expr)
     (let ((op (car expr)))
       (cond
         ;; (quote datum) — データをそのまま返す
         ((eq? op 'quote)
          (cadr expr))

         ;; (if cond then else) — 条件分岐
         ((eq? op 'if)
          (if (my-eval (cadr expr) env)
              (my-eval (caddr expr) env)
              (my-eval (cadddr expr) env)))

         ;; (def name value) または (def (name args...) body)
         ((eq? op 'def)
          (if (pair? (cadr expr))
              ;; (def (f x) body) => (def f (lambda (x) body))
              (let ((name (caadr expr))
                    (params (cdadr expr))
                    (body (cddr expr)))
                (set! *global-env*
                      (cons (cons name (list 'closure params body env))
                            *global-env*))
                name)
              ;; (def x value)
              (let ((name (cadr expr))
                    (val (my-eval (caddr expr) env)))
                (set! *global-env*
                      (cons (cons name val) *global-env*))
                name)))

         ;; (lambda (params...) body...) — クロージャを作る
         ((eq? op 'lambda)
          (let ((params (cadr expr))
                (body (cddr expr)))
            (list 'closure params body env)))

         ;; (begin expr1 expr2 ...) — 順に評価し最後の値を返す
         ((eq? op 'begin)
          (eval-sequence (cdr expr) env))

         ;; それ以外 — 関数呼び出し
         (else
          (let ((proc (my-eval op env))
                (args (map (lambda (a) (my-eval a env))
                           (cdr expr))))
            (my-apply proc args))))))

    (else
     (error "Unknown expression:" expr))))

;;; ===== 関数適用 =====

(define (my-apply proc args)
  (cond
    ;; 組み込み関数（ホストSchemeの関数をそのまま使う）
    ((procedure? proc)
     (apply proc args))

    ;; ユーザー定義関数（クロージャ）
    ((and (pair? proc) (eq? (car proc) 'closure))
     (let ((params (cadr proc))
           (body (caddr proc))
           (closed-env (cadddr proc)))
       (let ((new-env (extend-env params args closed-env)))
         (eval-sequence body new-env))))

    (else
     (error "Not a function:" proc))))

;;; ===== REPL =====

(define (my-repl)
  (set! *global-env* (make-global-env))
  (let loop ()
    (display "mini> ")
    (flush)
    (let ((input (read)))
      (if (eof-object? input)
          (begin (newline) (display "Bye!") (newline))
          (begin
            (display (my-eval input '()))
            (newline)
            (loop))))))

;;; ===== 補助関数 =====

;; 式の列を順に評価し、最後の値を返す
(define (eval-sequence exprs env)
  (if (null? (cdr exprs))
      (my-eval (car exprs) env)
      (begin
        (my-eval (car exprs) env)
        (eval-sequence (cdr exprs) env))))

;;; ===== 環境 =====

;; ローカル環境に束縛を追加する（純粋関数的）
(define (extend-env names vals base-env)
  (if (null? names)
      base-env
      (cons (cons (car names) (car vals))
            (extend-env (cdr names) (cdr vals) base-env))))

;; 変数を検索する: まずローカル環境、次にグローバル環境
(define (lookup-env sym env)
  (let ((found (assq sym env)))
    (if found
        (cdr found)
        (let ((global-found (assq sym *global-env*)))
          (if global-found
              (cdr global-found)
              (error "Undefined variable:" sym))))))

;;; ===== 初期環境 =====

(define (make-global-env)
  (list
    ;; 算術演算
    (cons '+ +) (cons '- -) (cons '* *) (cons '/ /)
    ;; 比較
    (cons '= =) (cons '< <) (cons '> >) (cons '<= <=) (cons '>= >=)
    ;; リスト操作
    (cons 'car car) (cons 'cdr cdr) (cons 'cons cons)
    (cons 'list list) (cons 'null? null?) (cons 'pair? pair?)
    ;; 入出力
    (cons 'display display) (cons 'newline newline)
    ;; その他
    (cons 'not not) (cons 'eq? eq?)))
