use crate::diagnostics::Diagnostic;
use crate::types::{Token, TokenKind};

const KEYWORDS: [(&str, TokenKind); 16] = [
    ("scene", TokenKind::Scene),
    ("show", TokenKind::Show),
    ("background", TokenKind::Background),
    ("music", TokenKind::Music),
    ("sound", TokenKind::Sound),
    ("voice", TokenKind::Voice),
    ("remove", TokenKind::Remove),
    ("clear", TokenKind::Clear),
    ("choice", TokenKind::ChoiceBlock),
    ("commit", TokenKind::Commit),
    ("jump", TokenKind::Jump),
    ("if", TokenKind::If),
    ("else", TokenKind::Else),
    ("call", TokenKind::Call),
    ("set", TokenKind::Set),
    ("add", TokenKind::Add),
];

pub fn keyword(word: &str) -> Option<TokenKind> {
    KEYWORDS
        .iter()
        .find(|(name, _)| *name == word)
        .map(|&(_, kind)| kind)
}

pub fn keyword_name(kind: TokenKind) -> Option<&'static str> {
    KEYWORDS
        .iter()
        .find(|(_, k)| *k == kind)
        .map(|&(name, _)| name)
}

pub fn keywords() -> impl Iterator<Item = &'static str> {
    KEYWORDS.iter().map(|&(name, _)| name)
}

pub fn is_keyword(word: &str) -> bool {
    keyword(word).is_some()
}

pub fn tokenize(input: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();

    for (line_idx, line) in input.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let line_number = line_idx + 1;
        let leading = &line[..line.len() - line.trim_start().len()];
        if leading.contains('\t') {
            diagnostics.push(Diagnostic::error(
                line_number,
                "tabs aren't allowed in indentation; use spaces",
            ));
            continue;
        }

        let first = trimmed.split_whitespace().next().unwrap_or_default();
        let word = first.strip_suffix(':').unwrap_or(first);

        let (kind, payload) = match keyword(word) {
            Some(kind) => (kind, trimmed[word.len()..].trim()),
            None if trimmed.starts_with('"') => (quoted_kind(trimmed), trimmed),
            None => (TokenKind::Dialogue, trimmed),
        };

        tokens.push(Token {
            kind,
            indent: leading.len(),
            payload: payload.to_string(),
            line: line_number,
        });
    }

    (tokens, diagnostics)
}

fn quoted_kind(line: &str) -> TokenKind {
    match scan_string(line) {
        Ok((_, used)) if line[used..].trim() == ":" => TokenKind::ChoiceOption,
        _ => TokenKind::Narration,
    }
}

pub fn scan_string(input: &str) -> Result<(String, usize), String> {
    let mut chars = input.char_indices();
    if !matches!(chars.next(), Some((_, '"'))) {
        return Err("expected a string in double quotes".to_string());
    }

    let mut text = String::new();
    while let Some((index, c)) = chars.next() {
        match c {
            '"' => return Ok((text, index + 1)),
            '\\' => match chars.next() {
                Some((_, '"')) => text.push('"'),
                Some((_, '\\')) => text.push('\\'),
                Some((_, other)) => {
                    return Err(format!(
                        "unknown escape `\\{}` (only `\\\"` and `\\\\` are allowed)",
                        other
                    ));
                }
                None => break,
            },
            _ => text.push(c),
        }
    }

    Err("unterminated string (missing the closing `\"`)".to_string())
}
