// src/main.rs — Chapter 10: 組み込み関数

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ===== トークン（Chapter 6） =====

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LParen,
    RParen,
    Number(f64),
    Str(String),
    Bool(bool),
    Symbol(String),
    Quote,
    Dot,
}

// ===== S式 / 値（Chapter 7-8） =====

#[derive(Debug, Clone)]
enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Symbol(String),
    List(Vec<Value>),
    DottedList(Vec<Value>, Box<Value>),
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
            (Value::DottedList(a1, a2), Value::DottedList(b1, b2)) => a1 == b1 && a2 == b2,
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
            Value::DottedList(elems, last) => {
                write!(f, "(")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, " . {}", last)?;
                write!(f, ")")
            }
            Value::Closure { .. } => write!(f, "#<closure>"),
            Value::BuiltinFunc(name) => write!(f, "#<builtin:{}>", name),
        }
    }
}

// ===== 環境（Chapter 8） =====

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

// ===== 字句解析（Chapter 6） =====

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
                    if sym == "." {
                        tokens.push(Token::Dot);
                    } else {
                        tokens.push(Token::Symbol(sym));
                    }
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

// ===== 構文解析（Chapter 7） =====

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
                if rest[0] == Token::Dot {
                    if list.is_empty() {
                        return Err("Unexpected '.' at start of list".to_string());
                    }
                    rest = &rest[1..];
                    let (tail, remaining) = parse(rest)?;
                    rest = remaining;
                    if rest.is_empty() || rest[0] != Token::RParen {
                        return Err("Expected ')' after dotted pair tail".to_string());
                    }
                    rest = &rest[1..];
                    return match tail {
                        Value::Nil => Ok((Value::List(list), rest)),
                        Value::List(mut elems) => {
                            list.append(&mut elems);
                            Ok((Value::List(list), rest))
                        }
                        Value::DottedList(mut elems, last) => {
                            list.append(&mut elems);
                            Ok((Value::DottedList(list, last), rest))
                        }
                        other => Ok((
                            Value::DottedList(list, Box::new(other)),
                            rest,
                        )),
                    };
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

        Token::Dot => Err("Unexpected '.'".to_string()),

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

// ===== 評価器（Chapter 8） =====

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
        Value::DottedList(..) => {
            Err("Cannot evaluate improper list (dotted pair)".to_string())
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

fn apply_builtin(name: &str, args: &[Value]) -> Result<Value, String> {
    match name {
        "+" => numeric_op(args, |a, b| a + b, 0.0),
        "-" => {
            if args.is_empty() {
                return Err("- requires at least 1 argument".to_string());
            }
            numeric_op(args, |a, b| a - b, 0.0)
        }
        "*" => numeric_op(args, |a, b| a * b, 1.0),
        "/" => {
            if args.is_empty() {
                return Err("/ requires at least 1 argument".to_string());
            }
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
        "pair?" => {
            let result = match args.first() {
                Some(Value::List(v)) if !v.is_empty() => true,
                Some(Value::DottedList(..)) => true,
                _ => false,
            };
            Ok(Value::Bool(result))
        }
        "list?" => Ok(Value::Bool(matches!(
            args.first(),
            Some(Value::List(_)) | Some(Value::Nil)
        ))),
        "null?" | "number?" | "string?"
        | "boolean?" | "symbol?" | "procedure?" => {
            if args.len() != 1 {
                return Err(format!(
                    "{} requires exactly 1 argument", name
                ));
            }
            let val = &args[0];
            let result = match name {
                "null?" => matches!(val, Value::Nil),
                "number?" => matches!(val, Value::Number(_)),
                "string?" => matches!(val, Value::Str(_)),
                "boolean?" => matches!(val, Value::Bool(_)),
                "symbol?" => matches!(val, Value::Symbol(_)),
                "procedure?" => matches!(
                    val, Value::Closure { .. } | Value::BuiltinFunc(_)
                ),
                _ => unreachable!(),
            };
            Ok(Value::Bool(result))
        }
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
        Value::DottedList(elems, _) => Ok(elems[0].clone()),
        _ => Err("car: argument must be a pair".to_string()),
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
        Value::DottedList(elems, last) => {
            if elems.len() == 1 {
                Ok(*last.clone())
            } else {
                Ok(Value::DottedList(elems[1..].to_vec(), last.clone()))
            }
        }
        _ => Err("cdr: argument must be a pair".to_string()),
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
        Value::DottedList(elems, last) => {
            let mut new_list = vec![args[0].clone()];
            new_list.extend(elems.iter().cloned());
            Ok(Value::DottedList(new_list, last.clone()))
        }
        Value::Nil => Ok(Value::List(vec![args[0].clone()])),
        _ => Ok(Value::DottedList(
            vec![args[0].clone()],
            Box::new(args[1].clone()),
        )),
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

fn make_global_env() -> EnvRef {
    let env = Env::new();
    let builtins = vec![
        "+", "-", "*", "/",
        "=", "<", ">", "<=", ">=",
        "car", "cdr", "cons", "list",
        "null?", "pair?", "list?", "number?", "string?",
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

fn main() {
    let env = make_global_env();

    let programs = vec![
        "(car '(1 2 3))",
        "(cdr '(1 2 3))",
        "(cons 0 '(1 2 3))",
        "(null? '())",
        "(number? 42)",
        r#"(string? "hello")"#,
        "(def (my-map f lst) (if (null? lst) '() (cons (f (car lst)) (my-map f (cdr lst)))))",
        "(my-map (lambda (x) (* x x)) '(1 2 3 4 5))",
        "(def (my-filter pred lst) (cond ((null? lst) '()) ((pred (car lst)) (cons (car lst) (my-filter pred (cdr lst)))) (else (my-filter pred (cdr lst)))))",
        "(my-filter (lambda (x) (> x 3)) '(1 2 3 4 5))",
    ];

    for input in programs {
        match tokenize(input) {
            Ok(tokens) => match parse_all(&tokens) {
                Ok(exprs) => {
                    for expr in &exprs {
                        match eval(expr, &env) {
                            Ok(val) => println!("{} => {}", input, val),
                            Err(e) => println!("{} => Error: {}", input, e),
                        }
                    }
                }
                Err(e) => println!("{} => Parse error: {}", input, e),
            },
            Err(e) => println!("{} => Tokenize error: {}", input, e),
        }
    }
}
