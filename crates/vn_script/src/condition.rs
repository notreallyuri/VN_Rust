use crate::parser::parse_identifier;
use crate::types::{Comparison, Condition, Value};

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Int(i32),
    Str(String),
    Op(Comparison),
    And,
    Or,
}

impl Token {
    fn describe(&self) -> String {
        match self {
            Token::Ident(name) => format!("`{}`", name),
            Token::Int(n) => format!("`{}`", n),
            Token::Str(text) => format!("`\"{}\"`", text),
            Token::Op(op) => format!("`{}`", op.symbol()),
            Token::And => "`&&`".to_string(),
            Token::Or => "`||`".to_string(),
        }
    }
}

pub fn parse_condition(payload: &str, line: usize) -> Condition {
    let expr = payload.trim();
    let expr = expr.strip_prefix("if").unwrap_or(expr).trim();
    let expr = expr
        .strip_suffix(':')
        .unwrap_or_else(|| panic!("`if` must end with ':' at line {}", line))
        .trim();

    if expr.is_empty() {
        panic!("`if` without a condition at line {}", line);
    }

    let tokens = tokenize(expr, line);
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
        line,
    };
    parser.any()
}

fn tokenize(expr: &str, line: usize) -> Vec<Token> {
    let chars: Vec<char> = expr.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        let two_char = match (c, next) {
            ('&', Some('&')) => Some(Token::And),
            ('|', Some('|')) => Some(Token::Or),
            ('=', Some('=')) => Some(Token::Op(Comparison::Equal)),
            ('!', Some('=')) => Some(Token::Op(Comparison::NotEqual)),
            ('>', Some('=')) => Some(Token::Op(Comparison::Gte)),
            ('<', Some('=')) => Some(Token::Op(Comparison::Lte)),
            _ => None,
        };

        if let Some(token) = two_char {
            tokens.push(token);
            i += 2;
            continue;
        }

        match c {
            '>' => {
                tokens.push(Token::Op(Comparison::Gt));
                i += 1;
            }
            '<' => {
                tokens.push(Token::Op(Comparison::Lt));
                i += 1;
            }
            '"' => {
                let end = chars[i + 1..]
                    .iter()
                    .position(|&c| c == '"')
                    .map(|offset| i + 1 + offset)
                    .unwrap_or_else(|| panic!("Unterminated string at line {}", line));
                tokens.push(Token::Str(chars[i + 1..end].iter().collect()));
                i = end + 1;
            }
            _ if c.is_ascii_digit() || (c == '-' && next.is_some_and(|n| n.is_ascii_digit())) => {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let n = text.parse().unwrap_or_else(|_| {
                    panic!("Integer `{}` is out of range at line {}", text, line)
                });
                tokens.push(Token::Int(n));
            }
            _ if c.is_ascii_alphanumeric() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                tokens.push(Token::Ident(chars[start..i].iter().collect()));
            }
            '=' => panic!("Use `==` to compare, not `=`, at line {}", line),
            _ => panic!("Unexpected `{}` in condition at line {}", c, line),
        }
    }

    tokens
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    line: usize,
}

impl Parser<'_> {
    fn any(&mut self) -> Condition {
        let mut parts = vec![self.all()];
        while self.eat(&Token::Or) {
            parts.push(self.all());
        }
        self.expect_end();
        collapse(parts, Condition::Any)
    }

    fn all(&mut self) -> Condition {
        let mut parts = vec![self.test()];
        while self.eat(&Token::And) {
            parts.push(self.test());
        }
        collapse(parts, Condition::All)
    }

    fn test(&mut self) -> Condition {
        let line = self.line;

        let var_id = match self.next() {
            Some(Token::Ident(name)) => parse_identifier(&name, line),
            Some(other) => panic!(
                "Expected a variable, got {} at line {}",
                other.describe(),
                line
            ),
            None => panic!(
                "Expected a variable at the end of the condition at line {}",
                line
            ),
        };

        let op = match self.next() {
            Some(Token::Op(op)) => op,
            _ => panic!(
                "Expected a comparison (== != < > <= >=) after `{}` at line {}",
                var_id, line
            ),
        };

        let value = match self.next() {
            Some(Token::Int(n)) => Value::Int(n),
            Some(Token::Str(text)) => Value::String(text),
            Some(Token::Ident(name)) if name == "true" => Value::Bool(true),
            Some(Token::Ident(name)) if name == "false" => Value::Bool(false),
            Some(Token::Ident(name)) => Value::Enum(parse_identifier(&name, line)),
            Some(other) => panic!(
                "Expected a value, got {} at line {}",
                other.describe(),
                line
            ),
            None => panic!("Expected a value after `{}` at line {}", op.symbol(), line),
        };

        let is_ordering = !matches!(op, Comparison::Equal | Comparison::NotEqual);
        if is_ordering && !matches!(value, Value::Int(_)) {
            panic!(
                "`{}` only compares integers, got `{}` at line {}",
                op.symbol(),
                value.literal(),
                line
            );
        }

        Condition::Test { var_id, op, value }
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        token
    }

    fn eat(&mut self, expected: &Token) -> bool {
        if self.tokens.get(self.pos) == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_end(&self) {
        if let Some(token) = self.tokens.get(self.pos) {
            panic!(
                "Expected `&&`, `||` or the end of the condition, got {} at line {}",
                token.describe(),
                self.line
            );
        }
    }
}

fn collapse(mut parts: Vec<Condition>, wrap: fn(Vec<Condition>) -> Condition) -> Condition {
    if parts.len() == 1 {
        parts.remove(0)
    } else {
        wrap(parts)
    }
}
