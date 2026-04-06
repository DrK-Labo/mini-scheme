// src/main.rs — Chapter 5: 字句解析

/// トークンの種類
#[derive(Debug, Clone, PartialEq)]
enum Token {
    LParen,           // (
    RParen,           // )
    Number(f64),      // 数値
    Str(String),      // 文字列
    Bool(bool),       // #t, #f
    Symbol(String),   // シンボル
    Quote,            // '
}

/// 文字列をトークン列に分解する
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

fn main() {
    let inputs = vec![
        "(+ 1 2)",
        "(def (square x) (* x x))",
        "(if #t \"yes\" \"no\")",
        "'(1 2 3)",
    ];

    for input in inputs {
        println!("Input: {}", input);
        match tokenize(input) {
            Ok(tokens) => {
                for token in &tokens {
                    println!("  {:?}", token);
                }
            }
            Err(e) => println!("  Error: {}", e),
        }
        println!();
    }
}
