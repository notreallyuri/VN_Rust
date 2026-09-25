use crate::diagnostics::Diagnostic;
use crate::lexer::scan_string;
use crate::parser::identifier;
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

pub fn parse_condition(expr: &str, line: usize) -> Result<Condition, Diagnostic> {
    let expr = expr.trim();
    if expr.is_empty() {
        return Err(Diagnostic::error(line, "`if` without a condition"));
    }

    let tokens = tokenize(expr).map_err(|message| Diagnostic::error(line, message))?;
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
        line,
    };
    parser.any()
}

fn tokenize(expr: &str) -> Result<Vec<Token>, String> {
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
                let rest: String = chars[i..].iter().collect();
                let (text, used) = scan_string(&rest)?;
                tokens.push(Token::Str(text));
                i += rest[..used].chars().count();
            }
            _ if c.is_ascii_digit() || (c == '-' && next.is_some_and(|n| n.is_ascii_digit())) => {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let n = text
                    .parse()
                    .map_err(|_| format!("integer `{}` is out of range (i32)", text))?;
                tokens.push(Token::Int(n));
            }
            _ if c.is_ascii_alphanumeric() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                tokens.push(Token::Ident(chars[start..i].iter().collect()));
            }
            '=' => return Err("use `==` to compare, not `=`".to_string()),
            _ => return Err(format!("unexpected `{}` in condition", c)),
        }
    }

    Ok(tokens)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    line: usize,
}

impl Parser<'_> {
    fn error(&self, message: String) -> Diagnostic {
        Diagnostic::error(self.line, message)
    }

    fn any(&mut self) -> Result<Condition, Diagnostic> {
        let mut parts = vec![self.all()?];
        while self.eat(&Token::Or) {
            parts.push(self.all()?);
        }
        self.expect_end()?;
        Ok(collapse(parts, Condition::Any))
    }

    fn all(&mut self) -> Result<Condition, Diagnostic> {
        let mut parts = vec![self.test()?];
        while self.eat(&Token::And) {
            parts.push(self.test()?);
        }
        Ok(collapse(parts, Condition::All))
    }

    fn test(&mut self) -> Result<Condition, Diagnostic> {
        let var_id = match self.next() {
            Some(Token::Ident(name)) => identifier(&name, "variable name", self.line)?,
            Some(other) => {
                return Err(self.error(format!("expected a variable, got {}", other.describe())));
            }
            None => {
                return Err(self.error("expected a variable at the end of the condition".into()));
            }
        };

        let op = match self.next() {
            Some(Token::Op(op)) => op,
            _ => {
                return Err(self.error(format!(
                    "expected a comparison (== != < > <= >=) after `{}`",
                    var_id
                )));
            }
        };

        let value = match self.next() {
            Some(Token::Int(n)) => Value::Int(n),
            Some(Token::Str(text)) => Value::String(text),
            Some(Token::Ident(name)) if name == "true" => Value::Bool(true),
            Some(Token::Ident(name)) if name == "false" => Value::Bool(false),
            Some(Token::Ident(name)) => Value::Enum(identifier(&name, "value", self.line)?),
            Some(other) => {
                return Err(self.error(format!("expected a value, got {}", other.describe())));
            }
            None => {
                return Err(self.error(format!("expected a value after `{}`", op.symbol())));
            }
        };

        let is_ordering = !matches!(op, Comparison::Equal | Comparison::NotEqual);
        if is_ordering && !matches!(value, Value::Int(_)) {
            return Err(self.error(format!(
                "`{}` only compares integers, got `{}`",
                op.symbol(),
                value.literal()
            )));
        }

        Ok(Condition::Test { var_id, op, value })
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

    fn expect_end(&self) -> Result<(), Diagnostic> {
        match self.tokens.get(self.pos) {
            Some(token) => Err(self.error(format!(
                "expected `&&`, `||` or the end of the condition, got {}",
                token.describe()
            ))),
            None => Ok(()),
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
