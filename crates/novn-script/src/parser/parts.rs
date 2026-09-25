use crate::diagnostics::Diagnostic;
use crate::lexer::{is_keyword, keywords, scan_string};
use crate::suggest::closest;

use super::statement::{keyword_speaker, words};
use crate::types::{Position, Transition, TransitionKind, Value};

pub(super) fn split_with(
    payload: &str,
    line: usize,
) -> Result<(Vec<&str>, Option<Transition>), Diagnostic> {
    let all = words(payload);
    let Some(at) = all.iter().position(|&word| word == "with") else {
        return Ok((all, None));
    };

    let error = |message: String| Diagnostic::error(line, message);
    let names = || TransitionKind::ALL.map(TransitionKind::name).join(", ");
    let transition = match all[at + 1..] {
        [] => return Err(error(format!("`with` needs a transition: {}", names()))),
        [name] => parse_transition(name, None, line)?,
        [name, seconds] => parse_transition(name, Some(seconds), line)?,
        [_, _, ref extra @ ..] => {
            return Err(error(format!(
                "`with` takes a transition and optionally its length in seconds; unexpected `{}`",
                extra.join(" ")
            )));
        }
    };
    Ok((all[..at].to_vec(), Some(transition)))
}

pub(super) fn parse_transition(
    name: &str,
    seconds: Option<&str>,
    line: usize,
) -> Result<Transition, Diagnostic> {
    let error = |message: String| Diagnostic::error(line, message);
    let kind = TransitionKind::from_name(name).ok_or_else(|| {
        let names = TransitionKind::ALL.map(TransitionKind::name);
        error(format!(
            "unknown transition `{}` (expected {}){}",
            name,
            names.join(", "),
            crate::did_you_mean(name, names)
        ))
    })?;

    let millis = match seconds {
        None => None,
        Some(text) => match text.parse::<f32>() {
            Ok(value) if value > 0.0 && value <= 30.0 => Some((value * 1000.0).round() as u32),
            _ => {
                return Err(error(format!(
                    "transition length must be a number of seconds between 0 and 30, got `{}`",
                    text
                )));
            }
        },
    };
    Ok(Transition { kind, millis })
}

pub(super) fn parse_position(name: &str, line: usize) -> Result<Position, Diagnostic> {
    Position::from_name(name).ok_or_else(|| {
        Diagnostic::error(
            line,
            format!(
                "unknown position `{}` (expected {})",
                name,
                position_names()
            ),
        )
    })
}

pub(super) fn position_names() -> String {
    Position::ALL.map(Position::name).join(", ")
}

pub fn parse_dialogue(payload: &str, line: usize) -> Result<(Option<String>, String), Diagnostic> {
    let error = |message: String| Diagnostic::error(line, message);

    let hint = keyword_hint(payload);

    let Some(quote) = payload.find('"') else {
        return Err(error(format!(
            "expected dialogue (`<character> \"<text>\"`) or a keyword, found `{}`{}",
            payload, hint
        )));
    };

    let speaker = match payload[..quote].trim() {
        "" => None,
        speaker => Some(speaker_id(speaker, line).map_err(|mut d| {
            d.message.push_str(&hint);
            d
        })?),
    };

    let (text, used) = scan_string(&payload[quote..]).map_err(error)?;
    let rest = payload[quote + used..].trim();
    if !rest.is_empty() {
        return Err(error(format!(
            "unexpected `{}` after the closing quote",
            rest
        )));
    }

    Ok((speaker, text))
}

pub(super) fn keyword_hint(payload: &str) -> String {
    let before_text = payload.find('"').map_or(payload, |quote| &payload[..quote]);
    let mut words = before_text.split_whitespace();
    let Some(first) = words.next() else {
        return String::new();
    };
    if payload.contains('"') && words.next().is_none() {
        return String::new();
    }

    let word = first.strip_suffix(':').unwrap_or(first);
    match closest(word, keywords()) {
        Some(keyword) => format!("; did you mean `{}`?", keyword),
        None => String::new(),
    }
}

pub(super) fn speaker_id(speaker: &str, line: usize) -> Result<String, Diagnostic> {
    if let Some(variable) = speaker
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
    {
        identifier(variable, "variable name", line)?;
        return Ok(speaker.to_string());
    }

    character_id(speaker, line)
}

pub(super) fn character_id(id: &str, line: usize) -> Result<String, Diagnostic> {
    if is_keyword(id) {
        return Err(Diagnostic::error(line, keyword_speaker(id)));
    }
    identifier(id, "character id", line)
}

pub fn parse_value(literal: &str, line: usize) -> Result<Value, Diagnostic> {
    let error = |message: String| Diagnostic::error(line, message);

    match literal {
        "" => Err(error("expected a value".into())),
        "true" => Ok(Value::Bool(true)),
        "false" => Ok(Value::Bool(false)),
        _ if literal.starts_with('"') => {
            let (text, used) = scan_string(literal).map_err(error)?;
            match literal[used..].trim() {
                "" => Ok(Value::String(text)),
                rest => Err(error(format!(
                    "unexpected `{}` after the closing quote",
                    rest
                ))),
            }
        }
        _ => match literal.parse::<i32>() {
            Ok(n) => Ok(Value::Int(n)),
            Err(_) if literal.starts_with(|c: char| c == '-' || c.is_ascii_digit()) => {
                Err(error(format!("`{}` is not a valid integer (i32)", literal)))
            }
            Err(_) => Ok(Value::Enum(identifier(literal, "value", line)?)),
        },
    }
}

pub fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub(crate) fn identifier(text: &str, what: &str, line: usize) -> Result<String, Diagnostic> {
    if text.is_empty() {
        return Err(Diagnostic::error(line, format!("missing {}", what)));
    }

    if !is_identifier(text) {
        return Err(Diagnostic::error(
            line,
            format!(
                "invalid {} `{}`: use lowercase letters, digits and `_`, not starting with a digit",
                what, text
            ),
        ));
    }

    Ok(text.to_string())
}
