use crate::diagnostics::Diagnostic;
use crate::suggest::did_you_mean;

pub const TAGS: [&str; 5] = ["b", "i", "color", "size", "w"];

const DEFAULT_PAUSE: f32 = 0.5;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub color: Option<u32>,
    pub size: Option<f32>,
    pub pause: f32,
}

impl Span {
    pub fn same_style(&self, other: &Span) -> bool {
        self.bold == other.bold
            && self.italic == other.italic
            && self.color == other.color
            && self.size == other.size
    }
}

#[derive(Debug, PartialEq)]
enum Piece<'a> {
    Text(&'a str),
    Literal(char),
    Open(&'a str, Option<&'a str>),
    Close(&'a str),
    Unclosed(&'a str),
}

fn pieces(text: &str) -> Vec<Piece<'_>> {
    let mut pieces = Vec::new();
    let mut rest = text;

    while let Some(at) = rest.find('[') {
        if at > 0 {
            pieces.push(Piece::Text(&rest[..at]));
        }
        let tail = &rest[at..];

        if let Some(after) = tail.strip_prefix("[[") {
            pieces.push(Piece::Literal('['));
            rest = after;
            continue;
        }

        let Some(close) = tail.find(']') else {
            pieces.push(Piece::Unclosed(tail));
            return pieces;
        };

        let body = &tail[1..close];
        match body.strip_prefix('/') {
            Some(name) => pieces.push(Piece::Close(name)),
            None => match body.split_once('=') {
                Some((name, value)) => pieces.push(Piece::Open(name, Some(value))),
                None => pieces.push(Piece::Open(body, None)),
            },
        }
        rest = &tail[close + 1..];
    }

    if !rest.is_empty() {
        pieces.push(Piece::Text(rest));
    }
    pieces
}

fn colour(value: &str) -> Option<u32> {
    let digits = value.trim().trim_start_matches('#');
    if digits.len() != 6 {
        return None;
    }
    u32::from_str_radix(digits, 16).ok()
}

#[derive(Default)]
struct Style {
    bold: usize,
    italic: usize,
    colors: Vec<u32>,
    sizes: Vec<f32>,
}

impl Style {
    fn apply(&self, span: &mut Span) {
        span.bold = self.bold > 0;
        span.italic = self.italic > 0;
        span.color = self.colors.last().copied();
        span.size = self.sizes.last().copied();
    }
}

pub fn parse(text: &str) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    let mut style = Style::default();
    let mut current = Span::default();
    style.apply(&mut current);

    let flush = |current: &mut Span, spans: &mut Vec<Span>| {
        if !current.text.is_empty() || current.pause > 0.0 {
            spans.push(std::mem::take(current));
        }
    };

    for piece in pieces(text) {
        match piece {
            Piece::Text(text) => current.text.push_str(text),
            Piece::Literal(c) => current.text.push(c),
            Piece::Unclosed(text) => current.text.push_str(text),
            Piece::Open(name, value) => match name {
                "b" | "i" | "color" | "size" => {
                    flush(&mut current, &mut spans);
                    match name {
                        "b" => style.bold += 1,
                        "i" => style.italic += 1,
                        "color" => {
                            if let Some(color) = value.and_then(colour) {
                                style.colors.push(color);
                            }
                        }
                        _ => {
                            if let Some(size) = value.and_then(|v| v.trim().parse::<f32>().ok()) {
                                style.sizes.push(size);
                            }
                        }
                    }
                    style.apply(&mut current);
                }
                "w" => {
                    flush(&mut current, &mut spans);
                    style.apply(&mut current);
                    current.pause = value
                        .and_then(|v| v.trim().parse::<f32>().ok())
                        .unwrap_or(DEFAULT_PAUSE);
                }
                _ => current.text.push_str(&format!("[{}]", name)),
            },
            Piece::Close(name) => {
                flush(&mut current, &mut spans);
                match name {
                    "b" => style.bold = style.bold.saturating_sub(1),
                    "i" => style.italic = style.italic.saturating_sub(1),
                    "color" => {
                        style.colors.pop();
                    }
                    "size" => {
                        style.sizes.pop();
                    }
                    _ => {}
                }
                style.apply(&mut current);
            }
        }
    }

    flush(&mut current, &mut spans);
    spans
}

pub fn plain(text: &str) -> String {
    parse(text).into_iter().map(|span| span.text).collect()
}

pub fn has_tags(text: &str) -> bool {
    pieces(text)
        .iter()
        .any(|piece| matches!(piece, Piece::Open(..) | Piece::Close(_)))
}

pub fn validate(text: &str, line: usize) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut open: Vec<&str> = Vec::new();

    for piece in pieces(text) {
        match piece {
            Piece::Unclosed(tag) => diagnostics.push(Diagnostic::error(
                line,
                format!("`{}` is missing its closing `]`", tag),
            )),
            Piece::Open(name, value) => {
                if !TAGS.contains(&name) {
                    diagnostics.push(Diagnostic::error(
                        line,
                        format!(
                            "unknown text tag `[{}]`{}",
                            name,
                            did_you_mean(name, TAGS.into_iter())
                        ),
                    ));
                    continue;
                }
                match (name, value) {
                    ("color", None) => diagnostics.push(Diagnostic::error(
                        line,
                        "`[color]` needs a value, like `[color=#c8a165]`",
                    )),
                    ("color", Some(value)) if colour(value).is_none() => {
                        diagnostics.push(Diagnostic::error(
                            line,
                            format!(
                                "`{}` isn't a colour; use six hex digits, like `#c8a165`",
                                value
                            ),
                        ))
                    }
                    ("size", None) => diagnostics.push(Diagnostic::error(
                        line,
                        "`[size]` needs a value, like `[size=28]`",
                    )),
                    ("size", Some(value)) if value.trim().parse::<f32>().is_err() => diagnostics
                        .push(Diagnostic::error(
                            line,
                            format!("`{}` isn't a text size, like `[size=28]`", value),
                        )),
                    ("w", Some(value)) if value.trim().parse::<f32>().is_err() => {
                        diagnostics.push(Diagnostic::error(
                            line,
                            format!("`{}` isn't a number of seconds, like `[w=0.5]`", value),
                        ))
                    }
                    _ => {}
                }
                if name != "w" {
                    open.push(name);
                }
            }
            Piece::Close(name) => {
                if !TAGS.contains(&name) {
                    diagnostics.push(Diagnostic::error(
                        line,
                        format!("unknown text tag `[/{}]`", name),
                    ));
                } else if open.pop() != Some(name) {
                    diagnostics.push(Diagnostic::error(
                        line,
                        format!("`[/{}]` doesn't close anything", name),
                    ));
                }
            }
            _ => {}
        }
    }

    for name in open {
        diagnostics.push(Diagnostic::warning(
            line,
            format!("`[{}]` is never closed; it ends with the line", name),
        ));
    }

    diagnostics
}
