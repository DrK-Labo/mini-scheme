// src/main.rs — Chapter 6: 構文解析

/// トークンの種類
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

/// S式（Schemeの値）
#[derive(Debug, Clone, PartialEq)]
enum Value {
    Number(f64),
    Str(String),
    Bool(bool),
    Symbol(String),
    List(Vec<Value>),
    Nil,
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
        }
    }
}

fn main() {
    let inputs = vec![
        "(+ 1 2)",
        "(def (square x) (* x x))",
        "(if #t \"yes\" \"no\")",
        "'(1 2 3)",
        "(+ 1 (* 2 3))",
        "()",
    ];

    for input in inputs {
        println!("Input: {}", input);
        match tokenize(input) {
            Ok(tokens) => match parse_all(&tokens) {
                Ok(exprs) => {
                    for expr in &exprs {
                        println!("  Parsed: {}", expr);
                    }
                }
                Err(e) => println!("  Parse error: {}", e),
            },
            Err(e) => println!("  Tokenize error: {}", e),
        }
        println!();
    }
}
