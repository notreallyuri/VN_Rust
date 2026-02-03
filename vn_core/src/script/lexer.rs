use super::types::{Token, TokenKind};

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for (line_idx, line) in input.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let indent = line.len() - line.trim_start().len();

        let (kind, payload) = match trimmed.split_whitespace().next() {
            Some("scene") => (TokenKind::Scene, trimmed.trim_start_matches("scene").trim()),
            Some("show") => (TokenKind::Show, trimmed.trim_start_matches("show").trim()),
            Some("remove") => (
                TokenKind::Remove,
                trimmed.trim_start_matches("remove").trim(),
            ),
            Some("clear") => (TokenKind::Clear, ""),
            Some("choice:") => (TokenKind::ChoiceBlock, ""),
            Some("jump") => (TokenKind::Jump, trimmed.trim_start_matches("jump").trim()),
            Some("if") => (TokenKind::If, trimmed),
            Some("else:") => (TokenKind::Else, ""),
            Some("call") => (TokenKind::Call, trimmed.trim_start_matches("call").trim()),
            Some("set") => (TokenKind::Set, trimmed.trim_start_matches("set").trim()),
            Some("add") => (TokenKind::Add, trimmed.trim_start_matches("add").trim()),

            _ => {
                if trimmed.starts_with('"') && trimmed.ends_with(':') {
                    (TokenKind::ChoiceOption, trimmed)
                } else if trimmed.starts_with('"') {
                    (TokenKind::Narration, trimmed)
                } else {
                    (TokenKind::Dialogue, trimmed)
                }
            }
        };

        tokens.push(Token {
            kind,
            indent,
            payload: payload.to_string(),
            line: line_idx + 1,
        });
    }

    tokens
}
