// src/main.rs — mini-scheme: Scheme interpreter in Rust

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;

// ===== トークン（Chapter 5） =====

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LParen,
    RParen,
    Number(f64),
    Str(String),
    Bool(bool),
    Symbol(String),
    Quote,
}

// ===== S式 / 値（Chapter 6-7） =====

#[derive(Debug, Clone)]
enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Symbol(String),
    List(Vec<Value>),
    Nil,
    Closure {
        params: Vec<String>,
        body: Vec<Value>,
        env: EnvRef,
    },
    BuiltinFunc(String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::BuiltinFunc(a), Value::BuiltinFunc(b)) => a == b,
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                if *n == (*n as i64) as f64 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Str(s) => write!(f, "\"{}\"", s),
            Value::Bool(true) => write!(f, "#t"),
            Value::Bool(false) => write!(f, "#f"),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::Nil => write!(f, "()"),
            Value::List(elems) => {
                write!(f, "(")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            }
            Value::Closure { .. } => write!(f, "#<closure>"),
            Value::BuiltinFunc(name) => write!(f, "#<builtin:{}>", name),
        }
    }
}

// ===== 環境（Chapter 7） =====

#[derive(Debug, Clone)]
struct Env {
    vars: HashMap<String, Value>,
    parent: Option<EnvRef>,
}

type EnvRef = Rc<RefCell<Env>>;

impl Env {
    fn new() -> EnvRef {
        Rc::new(RefCell::new(Env {
            vars: HashMap::new(),
            parent: None,
        }))
    }

    fn with_parent(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Env {
            vars: HashMap::new(),
            parent: Some(parent),
        }))
    }

    fn define(&mut self, name: String, val: Value) {
        self.vars.insert(name, val);
    }

    fn lookup(&self, name: &str) -> Result<Value, String> {
        if let Some(val) = self.vars.get(name) {
            Ok(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().lookup(name)
        } else {
            Err(format!("Undefined variable: {}", name))
        }
    }
}

// ===== 字句解析（Chapter 5） =====

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            ';' => {
                while let Some(&c) = chars.peek() {
                    if c == '\n' {
                        break;
                    }
                    chars.next();
                }
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '\'' => {
                tokens.push(Token::Quote);
                chars.next();
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('\\') => match chars.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some('\\') => s.push('\\'),
                            Some('"') => s.push('"'),
                            Some(c) => {
                                s.push('\\');
                                s.push(c);
                            }
                            None => return Err("Unexpected end of string".to_string()),
                        },
                        Some('"') => break,
                        Some(c) => s.push(c),
                        None => return Err("Unterminated string".to_string()),
                    }
                }
                tokens.push(Token::Str(s));
            }
            '#' => {
                chars.next();
                match chars.peek() {
                    Some(&'t') => {
                        chars.next();
                        tokens.push(Token::Bool(true));
                    }
                    Some(&'f') => {
                        chars.next();
                        tokens.push(Token::Bool(false));
                    }
                    _ => return Err("Expected #t or #f".to_string()),
                }
            }
            _ => {
                if ch.is_ascii_digit()
                    || (ch == '-' && is_digit_ahead(&mut chars))
                {
                    let num_str = read_while(&mut chars, |c| {
                        c.is_ascii_digit() || c == '.' || c == '-'
                    });
                    match num_str.parse::<f64>() {
                        Ok(n) => tokens.push(Token::Number(n)),
                        Err(_) => return Err(format!("Invalid number: {}", num_str)),
                    }
                } else {
                    let sym = read_while(&mut chars, |c| {
                        !c.is_whitespace()
                            && c != '('
                            && c != ')'
                            && c != '"'
                            && c != ';'
                    });
                    tokens.push(Token::Symbol(sym));
                }
            }
        }
    }

    Ok(tokens)
}

fn read_while<F>(chars: &mut std::iter::Peekable<std::str::Chars>, pred: F) -> String
where
    F: Fn(char) -> bool,
{
    let mut s = String::new();
    while let Some(&ch) = chars.peek() {
        if pred(ch) {
            s.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    s
}

fn is_digit_ahead(chars: &mut std::iter::Peekable<std::str::Chars>) -> bool {
    let mut clone = chars.clone();
    clone.next();
    match clone.peek() {
        Some(c) => c.is_ascii_digit(),
        None => false,
    }
}

// ===== 構文解析（Chapter 6） =====

fn parse(tokens: &[Token]) -> Result<(Value, &[Token]), String> {
    if tokens.is_empty() {
        return Err("Unexpected end of input".to_string());
    }

    match &tokens[0] {
        Token::LParen => {
            let mut list = Vec::new();
            let mut rest = &tokens[1..];

            loop {
                if rest.is_empty() {
                    return Err("Unexpected end of input: missing ')'".to_string());
                }
                if rest[0] == Token::RParen {
                    rest = &rest[1..];
                    break;
                }
                let (val, remaining) = parse(rest)?;
                list.push(val);
                rest = remaining;
            }

            if list.is_empty() {
                Ok((Value::Nil, rest))
            } else {
                Ok((Value::List(list), rest))
            }
        }

        Token::RParen => Err("Unexpected ')'".to_string()),

        Token::Quote => {
            let (val, rest) = parse(&tokens[1..])?;
            Ok((
                Value::List(vec![Value::Symbol("quote".to_string()), val]),
                rest,
            ))
        }

        Token::Number(n) => Ok((Value::Number(*n), &tokens[1..])),
        Token::Str(s) => Ok((Value::Str(s.clone()), &tokens[1..])),
        Token::Bool(b) => Ok((Value::Bool(*b), &tokens[1..])),
        Token::Symbol(s) => Ok((Value::Symbol(s.clone()), &tokens[1..])),
    }
}

fn parse_all(tokens: &[Token]) -> Result<Vec<Value>, String> {
    let mut results = Vec::new();
    let mut rest = tokens;

    while !rest.is_empty() {
        let (val, remaining) = parse(rest)?;
        results.push(val);
        rest = remaining;
    }

    Ok(results)
}

// ===== 評価器（Chapter 7-9） =====

fn eval(expr: &Value, env: &EnvRef) -> Result<Value, String> {
    match expr {
        Value::Number(_) | Value::Str(_) | Value::Bool(_) | Value::Nil => {
            Ok(expr.clone())
        }
        Value::Symbol(name) => env.borrow().lookup(name),
        Value::List(elems) => {
            if elems.is_empty() {
                return Ok(Value::Nil);
            }
            if let Value::Symbol(op) = &elems[0] {
                match op.as_str() {
                    "quote" => eval_quote(elems),
                    "if" => eval_if(elems, env),
                    "def" => eval_define(elems, env),
                    "lambda" => eval_lambda(elems, env),
                    "begin" => eval_begin(elems, env),
                    "set!" => eval_set(elems, env),
                    "cond" => eval_cond(elems, env),
                    "let" => eval_let(elems, env),
                    "and" => eval_and(elems, env),
                    _ => eval_call(elems, env),
                }
            } else {
                eval_call(elems, env)
            }
        }
        Value::Closure { .. } | Value::BuiltinFunc(_) => Ok(expr.clone()),
    }
}

fn is_truthy(val: &Value) -> bool {
    !matches!(val, Value::Bool(false))
}

fn eval_quote(elems: &[Value]) -> Result<Value, String> {
    if elems.len() != 2 {
        return Err("quote requires exactly 1 argument".to_string());
    }
    Ok(elems[1].clone())
}

fn eval_if(elems: &[Value], env: &EnvRef) -> Result<Value, String> {
    if elems.len() < 3 {
        return Err("if requires at least 2 arguments".to_string());
    }
    let cond_val = eval(&elems[1], env)?;
    if is_truthy(&cond_val) {
        eval(&elems[2], env)
    } else if elems.len() > 3 {
        eval(&elems[3], env)
    } else {
        Ok(Value::Nil)
    }
}

fn eval_define(elems: &[Value], env: &EnvRef) -> Result<Value, String> {
    match &elems[1] {
        Value::Symbol(name) => {
            if elems.len() != 3 {
                return Err("def requires a value".to_string());
            }
            let val = eval(&elems[2], env)?;
            env.borrow_mut().define(name.clone(), val);
            Ok(Value::Symbol(name.clone()))
        }
        Value::List(name_and_params) => {
            if name_and_params.is_empty() {
                return Err("def: empty function name".to_string());
            }
            let name = match &name_and_params[0] {
                Value::Symbol(s) => s.clone(),
                _ => return Err("def: function name must be a symbol".to_string()),
            };
            let params: Result<Vec<String>, String> = name_and_params[1..]
                .iter()
                .map(|p| match p {
                    Value::Symbol(s) => Ok(s.clone()),
                    _ => Err("def: parameter must be a symbol".to_string()),
                })
                .collect();
            let closure = Value::Closure {
                params: params?,
                body: elems[2..].to_vec(),
                env: Rc::clone(env),
            };
            env.borrow_mut().define(name.clone(), closure);
            Ok(Value::Symbol(name))
        }
        _ => Err("def: first argument must be a symbol or list".to_string()),
    }
}

fn eval_lambda(elems: &[Value], env: &EnvRef) -> Result<Value, String> {
    if elems.len() < 3 {
        return Err("lambda requires parameters and body".to_string());
    }
    let params: Result<Vec<String>, String> = match &elems[1] {
        Value::List(p) => p
            .iter()
            .map(|param| match param {
                Value::Symbol(s) => Ok(s.clone()),
                _ => Err("lambda: parameter must be a symbol".to_string()),
            })
            .collect(),
        Value::Nil => Ok(vec![]),
        _ => Err("lambda: parameters must be a list".to_string()),
    };
    Ok(Value::Closure {
        params: params?,
        body: elems[2..].to_vec(),
        env: Rc::clone(env),
    })
}

fn eval_begin(elems: &[Value], env: &EnvRef) -> Result<Value, String> {
    let mut result = Value::Nil;
    for expr in &elems[1..] {
        result = eval(expr, env)?;
    }
    Ok(result)
}

fn eval_begin_slice(exprs: &[Value], env: &EnvRef) -> Result<Value, String> {
    let mut result = Value::Nil;
    for expr in exprs {
        result = eval(expr, env)?;
    }
    Ok(result)
}

fn eval_set(elems: &[Value], env: &EnvRef) -> Result<Value, String> {
    if elems.len() != 3 {
        return Err("set! requires 2 arguments".to_string());
    }
    let name = match &elems[1] {
        Value::Symbol(s) => s.clone(),
        _ => return Err("set!: first argument must be a symbol".to_string()),
    };
    let val = eval(&elems[2], env)?;
    set_in_env(&name, val, env)?;
    Ok(Value::Nil)
}

fn set_in_env(name: &str, val: Value, env: &EnvRef) -> Result<(), String> {
    let mut env_ref = env.borrow_mut();
    if env_ref.vars.contains_key(name) {
        env_ref.vars.insert(name.to_string(), val);
        Ok(())
    } else if let Some(parent) = &env_ref.parent {
        let parent = Rc::clone(parent);
        drop(env_ref);
        set_in_env(name, val, &parent)
    } else {
        Err(format!("set!: undefined variable: {}", name))
    }
}

fn eval_cond(elems: &[Value], env: &EnvRef) -> Result<Value, String> {
    for clause in &elems[1..] {
        match clause {
            Value::List(parts) if parts.len() >= 2 => {
                if let Value::Symbol(s) = &parts[0] {
                    if s == "else" {
                        return eval_begin_slice(&parts[1..], env);
                    }
                }
                let cond_val = eval(&parts[0], env)?;
                if is_truthy(&cond_val) {
                    return eval_begin_slice(&parts[1..], env);
                }
            }
            _ => return Err("cond: invalid clause".to_string()),
        }
    }
    Ok(Value::Nil)
}

fn eval_let(elems: &[Value], env: &EnvRef) -> Result<Value, String> {
    if elems.len() < 3 {
        return Err("let requires bindings and body".to_string());
    }
    let bindings = match &elems[1] {
        Value::List(b) => b,
        Value::Nil => return eval_begin_slice(&elems[2..], env),
        _ => return Err("let: bindings must be a list".to_string()),
    };

    let new_env = Env::with_parent(Rc::clone(env));

    for binding in bindings {
        match binding {
            Value::List(pair) if pair.len() == 2 => {
                let name = match &pair[0] {
                    Value::Symbol(s) => s.clone(),
                    _ => return Err("let: binding name must be a symbol".to_string()),
                };
                let val = eval(&pair[1], env)?;
                new_env.borrow_mut().define(name, val);
            }
            _ => return Err("let: invalid binding".to_string()),
        }
    }

    eval_begin_slice(&elems[2..], &new_env)
}

fn eval_and(elems: &[Value], env: &EnvRef) -> Result<Value, String> {
    let mut result = Value::Bool(true);
    for expr in &elems[1..] {
        result = eval(expr, env)?;
        if !is_truthy(&result) {
            return Ok(Value::Bool(false));
        }
    }
    Ok(result)
}

fn eval_call(elems: &[Value], env: &EnvRef) -> Result<Value, String> {
    let func = eval(&elems[0], env)?;
    let args: Result<Vec<Value>, String> = elems[1..]
        .iter()
        .map(|a| eval(a, env))
        .collect();
    let args = args?;
    apply_func(&func, &args)
}

fn apply_func(func: &Value, args: &[Value]) -> Result<Value, String> {
    match func {
        Value::Closure { params, body, env } => {
            if params.len() != args.len() {
                return Err(format!(
                    "Expected {} arguments, got {}",
                    params.len(),
                    args.len()
                ));
            }
            let new_env = Env::with_parent(Rc::clone(env));
            for (name, val) in params.iter().zip(args.iter()) {
                new_env.borrow_mut().define(name.clone(), val.clone());
            }
            let mut result = Value::Nil;
            for expr in body {
                result = eval(expr, &new_env)?;
            }
            Ok(result)
        }
        Value::BuiltinFunc(name) => apply_builtin(name, args),
        _ => Err(format!("Not a function: {}", func)),
    }
}

// ===== 組み込み関数（Chapter 9） =====

fn apply_builtin(name: &str, args: &[Value]) -> Result<Value, String> {
    match name {
        "+" => numeric_op(args, |a, b| a + b, 0.0),
        "-" => numeric_op(args, |a, b| a - b, 0.0),
        "*" => numeric_op(args, |a, b| a * b, 1.0),
        "/" => {
            for arg in &args[1..] {
                if let Value::Number(n) = arg {
                    if *n == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                }
            }
            numeric_op(args, |a, b| a / b, 1.0)
        }
        "=" => compare_op(args, |a, b| a == b),
        "<" => compare_op(args, |a, b| a < b),
        ">" => compare_op(args, |a, b| a > b),
        "<=" => compare_op(args, |a, b| a <= b),
        ">=" => compare_op(args, |a, b| a >= b),
        "car" => builtin_car(args),
        "cdr" => builtin_cdr(args),
        "cons" => builtin_cons(args),
        "list" => Ok(if args.is_empty() {
            Value::Nil
        } else {
            Value::List(args.to_vec())
        }),
        "null?" => Ok(Value::Bool(matches!(args.first(), Some(Value::Nil)))),
        "pair?" => Ok(Value::Bool(matches!(
            args.first(),
            Some(Value::List(v)) if !v.is_empty()
        ))),
        "number?" => Ok(Value::Bool(matches!(args.first(), Some(Value::Number(_))))),
        "string?" => Ok(Value::Bool(matches!(args.first(), Some(Value::Str(_))))),
        "boolean?" => Ok(Value::Bool(matches!(args.first(), Some(Value::Bool(_))))),
        "symbol?" => Ok(Value::Bool(matches!(args.first(), Some(Value::Symbol(_))))),
        "procedure?" => Ok(Value::Bool(matches!(
            args.first(),
            Some(Value::Closure { .. }) | Some(Value::BuiltinFunc(_))
        ))),
        "eq?" => builtin_eq(args),
        "equal?" => builtin_equal(args),
        "not" => {
            if args.len() != 1 {
                return Err("not requires exactly 1 argument".to_string());
            }
            Ok(Value::Bool(!is_truthy(&args[0])))
        }
        "display" => {
            if args.len() != 1 {
                return Err("display requires exactly 1 argument".to_string());
            }
            match &args[0] {
                Value::Str(s) => print!("{}", s),
                other => print!("{}", other),
            }
            Ok(Value::Nil)
        }
        "newline" => {
            println!();
            Ok(Value::Nil)
        }
        _ => Err(format!("Unknown builtin function: {}", name)),
    }
}

fn numeric_op<F>(args: &[Value], op: F, identity: f64) -> Result<Value, String>
where
    F: Fn(f64, f64) -> f64,
{
    if args.is_empty() {
        return Ok(Value::Number(identity));
    }
    let first = match &args[0] {
        Value::Number(n) => *n,
        _ => return Err("Expected a number".to_string()),
    };
    if args.len() == 1 {
        return Ok(Value::Number(op(identity, first)));
    }
    let result = args[1..].iter().try_fold(first, |acc, arg| match arg {
        Value::Number(n) => Ok(op(acc, *n)),
        _ => Err("Expected a number".to_string()),
    })?;
    Ok(Value::Number(result))
}

fn compare_op<F>(args: &[Value], op: F) -> Result<Value, String>
where
    F: Fn(f64, f64) -> bool,
{
    if args.len() != 2 {
        return Err("Comparison requires exactly 2 arguments".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(op(*a, *b))),
        _ => Err("Comparison requires numbers".to_string()),
    }
}

fn builtin_car(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("car requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::List(elems) if !elems.is_empty() => Ok(elems[0].clone()),
        _ => Err("car: argument must be a non-empty list".to_string()),
    }
}

fn builtin_cdr(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("cdr requires exactly 1 argument".to_string());
    }
    match &args[0] {
        Value::List(elems) if !elems.is_empty() => {
            if elems.len() == 1 {
                Ok(Value::Nil)
            } else {
                Ok(Value::List(elems[1..].to_vec()))
            }
        }
        _ => Err("cdr: argument must be a non-empty list".to_string()),
    }
}

fn builtin_cons(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("cons requires exactly 2 arguments".to_string());
    }
    match &args[1] {
        Value::List(elems) => {
            let mut new_list = vec![args[0].clone()];
            new_list.extend(elems.iter().cloned());
            Ok(Value::List(new_list))
        }
        Value::Nil => Ok(Value::List(vec![args[0].clone()])),
        _ => Ok(Value::List(vec![args[0].clone(), args[1].clone()])),
    }
}

fn builtin_eq(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("eq? requires exactly 2 arguments".to_string());
    }
    let result = match (&args[0], &args[1]) {
        (Value::Symbol(a), Value::Symbol(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        _ => false,
    };
    Ok(Value::Bool(result))
}

fn builtin_equal(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("equal? requires exactly 2 arguments".to_string());
    }
    Ok(Value::Bool(args[0] == args[1]))
}

// ===== グローバル環境 =====

fn make_global_env() -> EnvRef {
    let env = Env::new();
    let builtins = vec![
        "+", "-", "*", "/",
        "=", "<", ">", "<=", ">=",
        "car", "cdr", "cons", "list",
        "null?", "pair?", "number?", "string?",
        "boolean?", "symbol?", "procedure?",
        "eq?", "equal?", "not",
        "display", "newline",
    ];
    for name in builtins {
        env.borrow_mut()
            .define(name.to_string(), Value::BuiltinFunc(name.to_string()));
    }
    env
}

// ===== REPL（Chapter 10） =====

fn run(input: &str, env: &EnvRef) -> Result<Value, String> {
    let tokens = tokenize(input)?;
    let exprs = parse_all(&tokens)?;

    let mut result = Value::Nil;
    for expr in &exprs {
        result = eval(expr, env)?;
    }
    Ok(result)
}

fn is_balanced(input: &str) -> bool {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
    }

    depth == 0 && !in_string
}

/// 式を評価して文字列を返すヘルパー（テスト・外部から利用）
#[allow(dead_code)]
fn eval_to_string(input: &str) -> Result<String, String> {
    let env = make_global_env();
    let val = run(input, &env)?;
    Ok(format!("{}", val))
}

fn main() {
    let env = make_global_env();

    println!("mini-scheme v0.1.0");
    println!("Type (exit) to quit.\n");

    let mut buffer = String::new();

    loop {
        if buffer.is_empty() {
            print!("mini> ");
        } else {
            print!("...   ");
        }
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => {
                println!("\nBye!");
                break;
            }
            Ok(_) => {
                buffer.push_str(&line);

                if buffer.trim() == "(exit)" {
                    println!("Bye!");
                    break;
                }

                if !is_balanced(&buffer) {
                    continue;
                }

                let input = buffer.trim().to_string();
                buffer.clear();

                if input.is_empty() {
                    continue;
                }

                match run(&input, &env) {
                    Ok(Value::Nil) => {}
                    Ok(val) => println!("{}", val),
                    Err(e) => println!("Error: {}", e),
                }
            }
            Err(e) => {
                println!("Read error: {}", e);
                break;
            }
        }
    }
}

// ===== テスト =====

#[cfg(test)]
mod tests {
    use super::*;

    // ヘルパー: 入力を評価して Value を返す
    fn eval_input(input: &str) -> Result<Value, String> {
        let env = make_global_env();
        run(input, &env)
    }

    // ヘルパー: 複数式を同一環境で評価し最後の値を返す
    fn eval_program(inputs: &[&str]) -> Result<Value, String> {
        let env = make_global_env();
        let mut result = Value::Nil;
        for input in inputs {
            result = run(input, &env)?;
        }
        Ok(result)
    }

    // ヘルパー: 評価結果の Display 文字列を返す
    fn eval_str(input: &str) -> String {
        format!("{}", eval_input(input).unwrap())
    }

    // ヘルパー: 複数式の最終結果の Display 文字列を返す
    fn eval_program_str(inputs: &[&str]) -> String {
        format!("{}", eval_program(inputs).unwrap())
    }

    // ===== Tokenizer =====

    #[test]
    fn tokenize_integer() {
        let tokens = tokenize("42").unwrap();
        assert_eq!(tokens, vec![Token::Number(42.0)]);
    }

    #[test]
    fn tokenize_negative_number() {
        let tokens = tokenize("-7").unwrap();
        assert_eq!(tokens, vec![Token::Number(-7.0)]);
    }

    #[test]
    fn tokenize_float() {
        let tokens = tokenize("3.14").unwrap();
        assert_eq!(tokens, vec![Token::Number(3.14)]);
    }

    #[test]
    fn tokenize_string() {
        let tokens = tokenize(r#""hello""#).unwrap();
        assert_eq!(tokens, vec![Token::Str("hello".to_string())]);
    }

    #[test]
    fn tokenize_string_escape() {
        let tokens = tokenize(r#""a\nb""#).unwrap();
        assert_eq!(tokens, vec![Token::Str("a\nb".to_string())]);
    }

    #[test]
    fn tokenize_booleans() {
        let tokens = tokenize("#t #f").unwrap();
        assert_eq!(tokens, vec![Token::Bool(true), Token::Bool(false)]);
    }

    #[test]
    fn tokenize_symbol() {
        let tokens = tokenize("foo").unwrap();
        assert_eq!(tokens, vec![Token::Symbol("foo".to_string())]);
    }

    #[test]
    fn tokenize_s_expression() {
        let tokens = tokenize("(+ 1 2)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Symbol("+".to_string()),
                Token::Number(1.0),
                Token::Number(2.0),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn tokenize_comment() {
        let tokens = tokenize("; this is a comment\n42").unwrap();
        assert_eq!(tokens, vec![Token::Number(42.0)]);
    }

    #[test]
    fn tokenize_quote_sugar() {
        let tokens = tokenize("'x").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Quote, Token::Symbol("x".to_string())]
        );
    }

    #[test]
    fn tokenize_unterminated_string() {
        assert!(tokenize(r#""hello"#).is_err());
    }

    #[test]
    fn tokenize_invalid_hash() {
        assert!(tokenize("#x").is_err());
    }

    // ===== Parser =====

    #[test]
    fn parse_atom() {
        let tokens = tokenize("42").unwrap();
        let (val, rest) = parse(&tokens).unwrap();
        assert_eq!(format!("{}", val), "42");
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_empty_list_is_nil() {
        let tokens = tokenize("()").unwrap();
        let (val, _) = parse(&tokens).unwrap();
        assert_eq!(val, Value::Nil);
    }

    #[test]
    fn parse_nested_list() {
        let tokens = tokenize("(+ (* 2 3) 4)").unwrap();
        let (val, rest) = parse(&tokens).unwrap();
        assert_eq!(format!("{}", val), "(+ (* 2 3) 4)");
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_quote_sugar() {
        let tokens = tokenize("'(1 2)").unwrap();
        let (val, _) = parse(&tokens).unwrap();
        assert_eq!(format!("{}", val), "(quote (1 2))");
    }

    #[test]
    fn parse_all_multiple() {
        let tokens = tokenize("1 2 3").unwrap();
        let results = parse_all(&tokens).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn parse_missing_rparen() {
        let tokens = tokenize("(+ 1").unwrap();
        assert!(parse(&tokens).is_err());
    }

    #[test]
    fn parse_unexpected_rparen() {
        let tokens = tokenize(")").unwrap();
        assert!(parse(&tokens).is_err());
    }

    // ===== Evaluator: 自己評価 =====

    #[test]
    fn eval_number() {
        assert_eq!(eval_str("42"), "42");
    }

    #[test]
    fn eval_string() {
        assert_eq!(eval_str(r#""hello""#), r#""hello""#);
    }

    #[test]
    fn eval_bool() {
        assert_eq!(eval_str("#t"), "#t");
        assert_eq!(eval_str("#f"), "#f");
    }

    // ===== Evaluator: 算術 =====

    #[test]
    fn eval_add() {
        assert_eq!(eval_str("(+ 1 2 3)"), "6");
    }

    #[test]
    fn eval_add_identity() {
        assert_eq!(eval_str("(+)"), "0");
    }

    #[test]
    fn eval_mul() {
        assert_eq!(eval_str("(* 2 3 4)"), "24");
    }

    #[test]
    fn eval_mul_identity() {
        assert_eq!(eval_str("(*)"), "1");
    }

    #[test]
    fn eval_sub() {
        assert_eq!(eval_str("(- 10 3)"), "7");
    }

    #[test]
    fn eval_unary_minus() {
        assert_eq!(eval_str("(- 5)"), "-5");
    }

    #[test]
    fn eval_div() {
        assert_eq!(eval_str("(/ 10 2)"), "5");
    }

    #[test]
    fn eval_div_by_zero() {
        assert!(eval_input("(/ 1 0)").is_err());
    }

    // ===== Evaluator: 比較 =====

    #[test]
    fn eval_eq_numbers() {
        assert_eq!(eval_str("(= 3 3)"), "#t");
        assert_eq!(eval_str("(= 3 4)"), "#f");
    }

    #[test]
    fn eval_lt() {
        assert_eq!(eval_str("(< 1 2)"), "#t");
        assert_eq!(eval_str("(< 2 1)"), "#f");
    }

    #[test]
    fn eval_gt() {
        assert_eq!(eval_str("(> 2 1)"), "#t");
    }

    #[test]
    fn eval_lte() {
        assert_eq!(eval_str("(<= 2 2)"), "#t");
        assert_eq!(eval_str("(<= 3 2)"), "#f");
    }

    #[test]
    fn eval_gte() {
        assert_eq!(eval_str("(>= 2 2)"), "#t");
    }

    // ===== Evaluator: quote =====

    #[test]
    fn eval_quote() {
        assert_eq!(eval_str("(quote (1 2 3))"), "(1 2 3)");
    }

    #[test]
    fn eval_quote_sugar() {
        assert_eq!(eval_str("'(a b c)"), "(a b c)");
    }

    // ===== Evaluator: if =====

    #[test]
    fn eval_if_true() {
        assert_eq!(eval_str("(if #t 1 2)"), "1");
    }

    #[test]
    fn eval_if_false() {
        assert_eq!(eval_str("(if #f 1 2)"), "2");
    }

    #[test]
    fn eval_if_no_else() {
        assert_eq!(eval_str("(if #f 1)"), "()");
    }

    // ===== Evaluator: def =====

    #[test]
    fn eval_def_variable() {
        assert_eq!(eval_program_str(&["(def x 42)", "x"]), "42");
    }

    #[test]
    fn eval_def_function() {
        assert_eq!(
            eval_program_str(&["(def (square n) (* n n))", "(square 5)"]),
            "25"
        );
    }

    // ===== Evaluator: lambda =====

    #[test]
    fn eval_lambda_call() {
        assert_eq!(eval_str("((lambda (x) (+ x 1)) 10)"), "11");
    }

    #[test]
    fn eval_lambda_no_params() {
        assert_eq!(eval_str("((lambda () 42))"), "42");
    }

    // ===== Evaluator: クロージャ =====

    #[test]
    fn eval_closure_lexical_scope() {
        assert_eq!(
            eval_program_str(&[
                "(def (make-adder n) (lambda (x) (+ n x)))",
                "(def add5 (make-adder 5))",
                "(add5 10)"
            ]),
            "15"
        );
    }

    #[test]
    fn eval_closure_capture() {
        assert_eq!(
            eval_program_str(&[
                "(def (counter) (let ((n 0)) (lambda () (set! n (+ n 1)) n)))",
                "(def c (counter))",
                "(c)",
                "(c)",
                "(c)"
            ]),
            "3"
        );
    }

    // ===== Evaluator: begin =====

    #[test]
    fn eval_begin() {
        assert_eq!(eval_str("(begin 1 2 3)"), "3");
    }

    // ===== Evaluator: set! =====

    #[test]
    fn eval_set() {
        assert_eq!(
            eval_program_str(&["(def x 1)", "(set! x 42)", "x"]),
            "42"
        );
    }

    // ===== Evaluator: cond =====

    #[test]
    fn eval_cond() {
        assert_eq!(eval_str("(cond (#f 1) (#t 2) (else 3))"), "2");
    }

    #[test]
    fn eval_cond_else() {
        assert_eq!(eval_str("(cond (#f 1) (else 99))"), "99");
    }

    // ===== Evaluator: let =====

    #[test]
    fn eval_let() {
        assert_eq!(eval_str("(let ((x 2) (y 3)) (+ x y))"), "5");
    }

    #[test]
    fn eval_let_empty_bindings() {
        assert_eq!(eval_str("(let () 42)"), "42");
    }

    // ===== Evaluator: and =====

    #[test]
    fn eval_and_all_true() {
        assert_eq!(eval_str("(and 1 2 3)"), "3");
    }

    #[test]
    fn eval_and_short_circuit() {
        assert_eq!(eval_str("(and 1 #f 3)"), "#f");
    }

    #[test]
    fn eval_and_empty() {
        assert_eq!(eval_str("(and)"), "#t");
    }

    // ===== Builtin: car/cdr/cons =====

    #[test]
    fn eval_car() {
        assert_eq!(eval_str("(car '(1 2 3))"), "1");
    }

    #[test]
    fn eval_cdr() {
        assert_eq!(eval_str("(cdr '(1 2 3))"), "(2 3)");
    }

    #[test]
    fn eval_cdr_singleton() {
        assert_eq!(eval_str("(cdr '(1))"), "()");
    }

    #[test]
    fn eval_cons() {
        assert_eq!(eval_str("(cons 1 '(2 3))"), "(1 2 3)");
    }

    #[test]
    fn eval_cons_nil() {
        assert_eq!(eval_str("(cons 1 '())"), "(1)");
    }

    // ===== Builtin: list =====

    #[test]
    fn eval_list() {
        assert_eq!(eval_str("(list 1 2 3)"), "(1 2 3)");
    }

    #[test]
    fn eval_list_empty() {
        assert_eq!(eval_str("(list)"), "()");
    }

    // ===== Builtin: 型述語 =====

    #[test]
    fn eval_type_predicates() {
        assert_eq!(eval_str("(number? 42)"), "#t");
        assert_eq!(eval_str("(number? \"x\")"), "#f");
        assert_eq!(eval_str("(string? \"hi\")"), "#t");
        assert_eq!(eval_str("(boolean? #t)"), "#t");
        assert_eq!(eval_str("(symbol? 'x)"), "#t");
        assert_eq!(eval_str("(null? '())"), "#t");
        assert_eq!(eval_str("(null? '(1))"), "#f");
        assert_eq!(eval_str("(pair? '(1 2))"), "#t");
        assert_eq!(eval_str("(pair? '())"), "#f");
        assert_eq!(eval_str("(procedure? +)"), "#t");
        assert_eq!(eval_str("(procedure? 42)"), "#f");
    }

    // ===== Builtin: eq?/equal? =====

    #[test]
    fn eval_eq() {
        assert_eq!(eval_str("(eq? 'a 'a)"), "#t");
        assert_eq!(eval_str("(eq? 'a 'b)"), "#f");
        assert_eq!(eval_str("(eq? 1 1)"), "#t");
    }

    #[test]
    fn eval_equal() {
        assert_eq!(eval_str("(equal? '(1 2) '(1 2))"), "#t");
        assert_eq!(eval_str("(equal? '(1 2) '(1 3))"), "#f");
    }

    // ===== Builtin: not =====

    #[test]
    fn eval_not() {
        assert_eq!(eval_str("(not #f)"), "#t");
        assert_eq!(eval_str("(not #t)"), "#f");
        assert_eq!(eval_str("(not 0)"), "#f"); // 0 is truthy in Scheme
    }

    // ===== 再帰 =====

    #[test]
    fn eval_factorial() {
        assert_eq!(
            eval_program_str(&[
                "(def (factorial n) (if (= n 0) 1 (* n (factorial (- n 1)))))",
                "(factorial 10)"
            ]),
            "3628800"
        );
    }

    #[test]
    fn eval_fibonacci() {
        assert_eq!(
            eval_program_str(&[
                "(def (fib n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2)))))",
                "(fib 10)"
            ]),
            "55"
        );
    }

    #[test]
    fn eval_map_user_defined() {
        assert_eq!(
            eval_program_str(&[
                "(def (my-map f lst) (if (null? lst) '() (cons (f (car lst)) (my-map f (cdr lst)))))",
                "(my-map (lambda (x) (* x x)) '(1 2 3 4 5))"
            ]),
            "(1 4 9 16 25)"
        );
    }

    // ===== エラー =====

    #[test]
    fn error_undefined_variable() {
        assert!(eval_input("xyz").is_err());
    }

    #[test]
    fn error_not_a_function() {
        assert!(eval_input("(42 1 2)").is_err());
    }

    #[test]
    fn error_arity_mismatch() {
        let result = eval_program(&["(def (f x) x)", "(f 1 2)"]);
        assert!(result.is_err());
    }

    #[test]
    fn error_set_undefined() {
        assert!(eval_input("(set! z 99)").is_err());
    }

    // ===== is_balanced =====

    #[test]
    fn balanced_simple() {
        assert!(is_balanced("(+ 1 2)"));
        assert!(!is_balanced("(+ 1"));
        assert!(is_balanced(""));
    }

    #[test]
    fn balanced_string() {
        assert!(is_balanced(r#""hello""#));
        assert!(!is_balanced(r#""hello"#));
    }

    #[test]
    fn balanced_nested() {
        assert!(is_balanced("((a) (b (c)))"));
        assert!(!is_balanced("((a) (b (c))"));
    }

    // ===== Display =====

    #[test]
    fn display_integer() {
        assert_eq!(format!("{}", Value::Number(5.0)), "5");
    }

    #[test]
    fn display_float() {
        assert_eq!(format!("{}", Value::Number(3.14)), "3.14");
    }

    #[test]
    fn display_nil() {
        assert_eq!(format!("{}", Value::Nil), "()");
    }

    #[test]
    fn display_closure() {
        let val = Value::Closure {
            params: vec![],
            body: vec![],
            env: Env::new(),
        };
        assert_eq!(format!("{}", val), "#<closure>");
    }

    #[test]
    fn display_builtin() {
        assert_eq!(format!("{}", Value::BuiltinFunc("+".to_string())), "#<builtin:+>");
    }

    // ===== eval_to_string =====

    #[test]
    fn eval_to_string_basic() {
        assert_eq!(eval_to_string("(+ 2 3)").unwrap(), "5");
    }
}
