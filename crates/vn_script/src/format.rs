use crate::lexer::{keyword, scan_string};
use crate::types::TokenKind;

pub const INDENT_WIDTH: usize = 2;

const OPERATORS: [&str; 11] = [
    "&&", "||", "==", "!=", ">=", "<=", "+=", "-=", "=", ">", "<",
];

enum Line {
    Blank,
    Comment(String),
    Content { indent: usize, text: String },
}

pub fn format(source: &str) -> String {
    let lines: Vec<Line> = source.lines().map(classify).collect();
    let levels = levels(&lines);
    render(&lines, &levels)
}

fn classify(line: &str) -> Line {
    let text = line.trim();
    if text.is_empty() {
        Line::Blank
    } else if text.starts_with('#') {
        Line::Comment(text.to_string())
    } else {
        Line::Content {
            indent: line.len() - line.trim_start().len(),
            text: text.to_string(),
        }
    }
}

fn levels(lines: &[Line]) -> Vec<usize> {
    let mut levels = vec![0; lines.len()];
    let mut stack = vec![0];

    for (i, line) in lines.iter().enumerate() {
        let Line::Content { indent, .. } = line else {
            continue;
        };
        while stack.len() > 1 && indent < stack.last().unwrap() {
            stack.pop();
        }
        if indent > stack.last().unwrap() {
            stack.push(*indent);
        }
        levels[i] = stack.len() - 1;
    }

    let last_content = lines
        .iter()
        .rposition(|line| matches!(line, Line::Content { .. }));
    let mut below = last_content.map_or(0, |i| levels[i]);
    for i in (0..lines.len()).rev() {
        match lines[i] {
            Line::Content { .. } => below = levels[i],
            _ => levels[i] = below,
        }
    }

    levels
}

fn render(lines: &[Line], levels: &[usize]) -> String {
    let mut out: Vec<String> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        match line {
            Line::Blank => {
                if !out.is_empty() && !out.last().unwrap().is_empty() {
                    out.push(String::new());
                }
            }
            Line::Comment(text) => out.push(indent(levels[i]) + text),
            Line::Content { text, .. } => {
                let text = normalize(text);
                if first_word(&text) == "scene" {
                    separate(&mut out);
                }
                out.push(indent(levels[i]) + &text);
            }
        }
    }

    while out.last().is_some_and(String::is_empty) {
        out.pop();
    }
    if out.is_empty() {
        return String::new();
    }
    out.join("\n") + "\n"
}

fn indent(level: usize) -> String {
    " ".repeat(level * INDENT_WIDTH)
}

fn separate(out: &mut Vec<String>) {
    let mut at = out.len();
    while at > 0 && out[at - 1].trim_start().starts_with('#') {
        at -= 1;
    }
    if at > 0 && !out[at - 1].is_empty() {
        out.insert(at, String::new());
    }
}

fn first_word(text: &str) -> &str {
    text.split_whitespace().next().unwrap_or_default()
}

fn normalize(text: &str) -> String {
    let (body, colon) = match text.strip_suffix(':') {
        Some(body) => (body, true),
        None => (text, false),
    };

    let spaced = matches!(
        keyword(first_word(body)),
        Some(TokenKind::If | TokenKind::Set | TokenKind::Add)
    );

    let mut line = atoms(body, spaced).join(" ");
    if colon {
        line.push(':');
    }
    line
}

fn atoms(text: &str, spaced: bool) -> Vec<String> {
    let mut atoms = Vec::new();
    let mut at = 0;

    while at < text.len() {
        let rest = &text[at..];
        let c = rest.chars().next().unwrap();

        if c.is_whitespace() {
            at += c.len_utf8();
            continue;
        }

        if c == '"' {
            let used = match scan_string(rest) {
                Ok((_, used)) => used,
                Err(_) => rest.len(),
            };
            atoms.push(rest[..used].to_string());
            at += used;
            continue;
        }

        if spaced && let Some(operator) = OPERATORS.iter().find(|op| rest.starts_with(**op)) {
            atoms.push((*operator).to_string());
            at += operator.len();
            continue;
        }

        let mut end = rest.len();
        for (offset, c) in rest.char_indices().skip(1) {
            let operator = spaced && OPERATORS.iter().any(|op| rest[offset..].starts_with(op));
            if c.is_whitespace() || c == '"' || operator {
                end = offset;
                break;
            }
        }
        atoms.push(rest[..end].to_string());
        at += end;
    }

    atoms
}
