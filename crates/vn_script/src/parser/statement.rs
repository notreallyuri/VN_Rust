use crate::diagnostics::Diagnostic;
use crate::lexer::scan_string;
use crate::types::parser::{Node, Token, TokenKind};

use super::parts::{
    character_id, identifier, parse_dialogue, parse_position, parse_value, position_names,
    split_with,
};

pub(super) fn keyword_speaker(keyword: &str) -> String {
    format!("`{}` is a keyword and can't be a character id", keyword)
}

pub(super) fn option_text(token: &Token) -> Result<String, Diagnostic> {
    let (text, _) = scan_string(&token.payload).map_err(|e| Diagnostic::error(token.line, e))?;
    if text.trim().is_empty() {
        return Err(Diagnostic::error(token.line, "choice option text is empty"));
    }
    Ok(text)
}

pub(super) fn words(payload: &str) -> Vec<&str> {
    payload.split_whitespace().collect()
}

pub(super) fn parse_statement(token: &Token) -> Result<Node, Diagnostic> {
    let line = token.line;
    let error = |message: String| Diagnostic::error(line, message);

    match token.kind {
        TokenKind::Show => {
            let (words, transition) = split_with(&token.payload, line)?;
            let (character, image, position) = match words[..] {
                [character, image] => (character, image, None),
                [character, image, "at", position] => {
                    (character, image, Some(parse_position(position, line)?))
                }
                [_, _, "at"] => {
                    return Err(error(format!(
                        "`at` needs a position: {}",
                        position_names()
                    )));
                }
                [] => {
                    return Err(error(
                        "`show` needs a character and an image: `show <character> <image>`".into(),
                    ));
                }
                [character] => {
                    return Err(error(format!(
                        "`show {}` needs an image: `show {} <image>`",
                        character, character
                    )));
                }
                [_, _, ref extra @ ..] => {
                    return Err(error(format!(
                        "`show` takes a character, an image and optionally `at <position>` and `with <transition>`; unexpected `{}`",
                        extra.join(" ")
                    )));
                }
            };
            Ok(Node::Show {
                character: character_id(character, line)?,
                image: identifier(image, "image id", line)?,
                position,
                transition,
            })
        }

        TokenKind::Background => {
            let (words, transition) = split_with(&token.payload, line)?;
            let image = match words[..] {
                ["none"] => None,
                [image] => Some(identifier(image, "background id", line)?),
                [] => {
                    return Err(error(
                        "`background` needs an image: `background <image>` or `background none`"
                            .into(),
                    ));
                }
                [_, ref extra @ ..] => {
                    return Err(error(format!(
                        "`background` takes one image; unexpected `{}`",
                        extra.join(" ")
                    )));
                }
            };
            Ok(Node::Background { image, transition })
        }

        TokenKind::Music => match words(&token.payload)[..] {
            ["none"] => Ok(Node::Music { track: None }),
            [track] => Ok(Node::Music {
                track: Some(identifier(track, "music track", line)?),
            }),
            [] => Err(error(
                "`music` needs a track: `music <track>` or `music none`".into(),
            )),
            [_, ref extra @ ..] => Err(error(format!(
                "`music` takes one track; unexpected `{}`",
                extra.join(" ")
            ))),
        },

        TokenKind::Sound => match words(&token.payload)[..] {
            [id] => Ok(Node::Sound {
                id: identifier(id, "sound id", line)?,
            }),
            [] => Err(error("`sound` needs a sound: `sound <id>`".into())),
            [_, ref extra @ ..] => Err(error(format!(
                "`sound` takes one sound; unexpected `{}`",
                extra.join(" ")
            ))),
        },

        TokenKind::Voice => match words(&token.payload)[..] {
            [id] => Ok(Node::Voice {
                id: identifier(id, "voice id", line)?,
            }),
            [] => Err(error("`voice` needs a clip: `voice <id>`".into())),
            [_, ref extra @ ..] => Err(error(format!(
                "`voice` takes one clip; unexpected `{}`",
                extra.join(" ")
            ))),
        },

        TokenKind::Remove => {
            let (words, transition) = split_with(&token.payload, line)?;
            match words[..] {
                [character] => Ok(Node::Remove {
                    character: character_id(character, line)?,
                    transition,
                }),
                _ => Err(error(
                    "`remove` takes one character: `remove <character>`".into(),
                )),
            }
        }

        TokenKind::Clear => {
            let (words, transition) = split_with(&token.payload, line)?;
            if words.is_empty() {
                Ok(Node::Clear { transition })
            } else {
                Err(error(
                    "`clear` takes no arguments (use `remove <character>` for one character)"
                        .into(),
                ))
            }
        }

        TokenKind::Commit if token.payload.is_empty() => Ok(Node::Commit),
        TokenKind::Commit => Err(error("`commit` takes no arguments".into())),

        TokenKind::Jump => match words(&token.payload)[..] {
            [target] => Ok(Node::Jump {
                target: identifier(target, "scene id", line)?,
            }),
            _ => Err(error("`jump` takes one scene: `jump <scene>`".into())),
        },

        TokenKind::Call => match words(&token.payload)[..] {
            [command, ref args @ ..] => Ok(Node::Call {
                command: identifier(command, "command name", line)?,
                args: args.iter().map(|arg| arg.to_string()).collect(),
            }),
            [] => Err(error(
                "`call` needs a command: `call <command> [args...]`".into(),
            )),
        },

        TokenKind::Set => parse_set(&token.payload, line),

        TokenKind::Add => parse_add(&token.payload, line),

        TokenKind::Dialogue | TokenKind::Narration => {
            let (speaker, text) = parse_dialogue(&token.payload, line)?;
            Ok(Node::Dialogue { speaker, text })
        }

        TokenKind::Scene
        | TokenKind::ChoiceBlock
        | TokenKind::ChoiceOption
        | TokenKind::If
        | TokenKind::Else => unreachable!("block tokens are parsed by the block parser"),
    }
}

pub(super) fn parse_set(payload: &str, line: usize) -> Result<Node, Diagnostic> {
    let (var_id, literal) = payload
        .split_once('=')
        .ok_or_else(|| Diagnostic::error(line, "expected `set <variable> = <value>`"))?;

    Ok(Node::Set {
        var_id: identifier(var_id.trim(), "variable name", line)?,
        value: parse_value(literal.trim(), line)?,
    })
}

pub(super) fn parse_add(payload: &str, line: usize) -> Result<Node, Diagnostic> {
    let (var_id, sign, amount) = if let Some((var_id, amount)) = payload.split_once("+=") {
        (var_id, 1, amount)
    } else if let Some((var_id, amount)) = payload.split_once("-=") {
        (var_id, -1, amount)
    } else {
        return Err(Diagnostic::error(
            line,
            "expected `add <variable> += <integer>` or `-=`",
        ));
    };

    let amount: i32 = amount.trim().parse().map_err(|_| {
        Diagnostic::error(
            line,
            format!("`add` needs an integer amount, got `{}`", amount.trim()),
        )
    })?;

    let amount = amount
        .checked_mul(sign)
        .ok_or_else(|| Diagnostic::error(line, "`add` amount is out of range"))?;

    Ok(Node::Add {
        var_id: identifier(var_id.trim(), "variable name", line)?,
        amount,
    })
}
