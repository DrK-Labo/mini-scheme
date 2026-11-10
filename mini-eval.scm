;;;; mini-eval.scm — a Scheme interpreter written in Scheme

;;; Global environment (variables defined by def accumulate here)
(define *global-env* '())

;;; ===== Evaluator =====

(define (my-eval expr env)
  (cond
    ;; Numbers, strings, and booleans are self-evaluating
    ((number? expr) expr)
    ((string? expr) expr)
    ((boolean? expr) expr)

    ;; Symbols: look up the value in the environment
    ((symbol? expr)
     (lookup-env expr env))

    ;; Lists (special forms or function calls)
    ((pair? expr)
     (let ((op (car expr)))
       (cond
         ;; (quote datum) — return the datum as is
         ((eq? op 'quote)
          (cadr expr))

         ;; (if cond then else) — conditional
         ((eq? op 'if)
          (if (my-eval (cadr expr) env)
              (my-eval (caddr expr) env)
              (my-eval (cadddr expr) env)))

         ;; (def name value) or (def (name args...) body)
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

         ;; (lambda (params...) body...) — build a closure
         ((eq? op 'lambda)
          (let ((params (cadr expr))
                (body (cddr expr)))
            (list 'closure params body env)))

         ;; (begin expr1 expr2 ...) — evaluate in order, return last
         ((eq? op 'begin)
          (eval-sequence (cdr expr) env))

         ;; Otherwise — a function call
         (else
          (let ((proc (my-eval op env))
                (args (map (lambda (a) (my-eval a env))
                           (cdr expr))))
            (my-apply proc args))))))

    (else
     (error "Unknown expression:" expr))))

;;; ===== Function application =====

(define (my-apply proc args)
  (cond
    ;; Built-in functions (reuse the host Scheme's procedures)
    ((procedure? proc)
     (apply proc args))

    ;; User-defined functions (closures)
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

;;; ===== Helpers =====

;; Evaluate a sequence of expressions in order, return the last value
(define (eval-sequence exprs env)
  (if (null? (cdr exprs))
      (my-eval (car exprs) env)
      (begin
        (my-eval (car exprs) env)
        (eval-sequence (cdr exprs) env))))

;;; ===== Environments =====

;; Add bindings to a local environment (purely functional)
(define (extend-env names vals base-env)
  (if (null? names)
      base-env
      (cons (cons (car names) (car vals))
            (extend-env (cdr names) (cdr vals) base-env))))

;; Look up a variable: search the local environment first, then global
(define (lookup-env sym env)
  (let ((found (assq sym env)))
    (if found
        (cdr found)
        (let ((global-found (assq sym *global-env*)))
          (if global-found
              (cdr global-found)
              (error "Undefined variable:" sym))))))

;;; ===== Initial environment =====

(define (make-global-env)
  (list
    ;; Arithmetic
    (cons '+ +) (cons '- -) (cons '* *) (cons '/ /)
    ;; Comparison
    (cons '= =) (cons '< <) (cons '> >) (cons '<= <=) (cons '>= >=)
    ;; List operations
    (cons 'car car) (cons 'cdr cdr) (cons 'cons cons)
    (cons 'list list) (cons 'null? null?) (cons 'pair? pair?)
    ;; I/O
    (cons 'display display) (cons 'newline newline)
    ;; Other
    (cons 'not not) (cons 'eq? eq?)))
