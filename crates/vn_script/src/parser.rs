use crate::diagnostics::Diagnostic;
use crate::lexer::{is_keyword, keyword_name, keywords, scan_string};
use crate::suggest::closest;
use crate::types::parser::{ChoiceOption, Node, Stmt, Token, TokenKind};
use crate::types::{Position, Transition, TransitionKind, Value};

pub use crate::condition::parse_condition;

pub fn parse_program(tokens: &[Token]) -> (Vec<Stmt>, Vec<Diagnostic>) {
    let mut parser = Parser {
        tokens,
        diagnostics: Vec::new(),
    };
    let scenes = parser.program();
    (scenes, parser.diagnostics)
}

struct Parser<'a> {
    tokens: &'a [Token],
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    fn error(&mut self, line: usize, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::error(line, message));
    }

    fn report<T>(&mut self, result: Result<T, Diagnostic>) -> Option<T> {
        result.map_err(|d| self.diagnostics.push(d)).ok()
    }

    fn skip_nested(&self, start: usize) -> usize {
        let indent = self.tokens[start].indent;
        let mut i = start + 1;
        while i < self.tokens.len() && self.tokens[i].indent > indent {
            i += 1;
        }
        i
    }

    fn indented(&self, start: usize) -> Option<usize> {
        let parent = self.tokens[start].indent;
        self.tokens
            .get(start + 1)
            .map(|next| next.indent)
            .filter(|&indent| indent > parent)
    }

    fn body(&mut self, start: usize, what: &str) -> (Vec<Stmt>, usize) {
        match self.indented(start) {
            Some(indent) => self.blocks(start + 1, indent),
            None => {
                self.error(
                    self.tokens[start].line,
                    format!("{} needs an indented block below it", what),
                );
                (Vec::new(), start + 1)
            }
        }
    }

    fn header(&mut self, token: &'a Token, what: &str) -> Option<&'a str> {
        let header = token.payload.strip_suffix(':').map(str::trim);
        if header.is_none() {
            self.error(token.line, format!("{} must end with `:`", what));
        }
        header
    }

    fn program(&mut self) -> Vec<Stmt> {
        let mut scenes = Vec::new();
        let mut i = 0;

        while i < self.tokens.len() {
            let token = &self.tokens[i];

            if token.kind != TokenKind::Scene {
                if scenes.is_empty() || token.indent == 0 {
                    self.error(
                        token.line,
                        "this line is outside a scene; start the file with `scene <id>:`",
                    );
                    i = self.skip_nested(i);
                } else {
                    self.error(token.line, "unexpected indentation");
                    i += 1;
                    while i < self.tokens.len() && self.tokens[i].indent > 0 {
                        i += 1;
                    }
                }
                continue;
            }

            if token.indent > 0 {
                self.error(token.line, "`scene` can't be indented");
            }

            let (scene, next) = self.scene(i);
            scenes.extend(scene);
            i = next;
        }

        scenes
    }

    fn scene(&mut self, start: usize) -> (Option<Stmt>, usize) {
        let token = &self.tokens[start];

        let id = self.header(token, "`scene <id>`").and_then(|id| {
            let id = identifier(id, "scene id", token.line);
            self.report(id)
        });

        let (body, next) = match self.indented(start) {
            Some(indent) => self.blocks(start + 1, indent),
            None => {
                if let Some(id) = &id {
                    self.diagnostics.push(Diagnostic::warning(
                        token.line,
                        format!("scene '{}' is empty", id),
                    ));
                }
                (Vec::new(), start + 1)
            }
        };

        let scene = id.map(|id| Stmt {
            line: token.line,
            node: Node::Scene { id, body },
        });
        (scene, next)
    }

    fn blocks(&mut self, start: usize, indent: usize) -> (Vec<Stmt>, usize) {
        let mut nodes = Vec::new();
        let mut i = start;

        while i < self.tokens.len() {
            let token = &self.tokens[i];

            if token.indent < indent {
                break;
            }

            if token.indent > indent {
                self.error(token.line, "unexpected indentation");
                while i < self.tokens.len() && self.tokens[i].indent > indent {
                    i += 1;
                }
                continue;
            }

            let (node, next) = self.statement(i);
            nodes.extend(node.map(|node| Stmt {
                line: token.line,
                node,
            }));
            i = next;
        }

        (nodes, i)
    }

    fn statement(&mut self, i: usize) -> (Option<Node>, usize) {
        let token = &self.tokens[i];

        if let Some(keyword) = keyword_name(token.kind)
            && token.payload.starts_with('"')
        {
            self.error(token.line, keyword_speaker(keyword));
            return (None, self.skip_nested(i));
        }

        match token.kind {
            TokenKind::ChoiceBlock => self.choice(i),
            TokenKind::If => self.if_block(i),
            TokenKind::Else => {
                self.error(token.line, "`else:` without a matching `if` above it");
                (None, self.skip_nested(i))
            }
            TokenKind::ChoiceOption => {
                self.error(
                    token.line,
                    "choice option outside a `choice:` block (narration doesn't end with `:`)",
                );
                (None, self.skip_nested(i))
            }
            TokenKind::Scene => {
                self.error(
                    token.line,
                    "`scene` can't be indented (scenes can't be nested)",
                );
                (None, self.skip_nested(i))
            }
            _ => {
                let node = parse_statement(token);
                (self.report(node), i + 1)
            }
        }
    }

    fn if_block(&mut self, start: usize) -> (Option<Node>, usize) {
        let token = &self.tokens[start];

        let condition = self.header(token, "`if <condition>`").and_then(|expr| {
            let condition = parse_condition(expr, token.line);
            self.report(condition)
        });
        let (then_branch, mut next) = self.body(start, "`if`");

        let mut else_branch = Vec::new();
        if let Some(else_token) = self.tokens.get(next)
            && else_token.kind == TokenKind::Else
            && else_token.indent == token.indent
        {
            if else_token.payload != ":" {
                self.error(
                    else_token.line,
                    "write `else:` on its own line (for `else if`, put an `if` inside `else:`)",
                );
            }
            let (branch, after) = self.body(next, "`else:`");
            else_branch = branch;
            next = after;
        }

        let node = condition.map(|condition| Node::If {
            condition,
            then_branch,
            else_branch,
        });
        (node, next)
    }

    fn choice(&mut self, start: usize) -> (Option<Node>, usize) {
        let token = &self.tokens[start];

        let final_choice = match self.header(token, "`choice`") {
            Some("final") => true,
            Some("") | None => false,
            Some(other) => {
                self.error(
                    token.line,
                    format!(
                        "unknown choice modifier `{}` (expected `choice:` or `choice final:`)",
                        other
                    ),
                );
                false
            }
        };

        let Some(option_indent) = self.indented(start) else {
            self.error(token.line, "`choice:` needs at least one option below it");
            return (None, start + 1);
        };

        let mut options = Vec::new();
        let mut i = start + 1;

        while i < self.tokens.len() && self.tokens[i].indent > token.indent {
            let option = &self.tokens[i];

            if option.indent != option_indent {
                self.error(option.line, "unexpected indentation in `choice:` block");
                i = self.skip_nested(i);
                continue;
            }

            if option.kind != TokenKind::ChoiceOption {
                self.error(
                    option.line,
                    "expected a choice option: quoted text ending with `:`, like `\"Agree\":`",
                );
                i = self.skip_nested(i);
                continue;
            }

            let text = self.report(option_text(option));
            let (body, next) = self.body(i, "choice option");
            options.extend(text.map(|text| ChoiceOption {
                text,
                line: option.line,
                body,
            }));
            i = next;
        }

        let node = Node::ChoiceBlock {
            options,
            final_choice,
        };
        (Some(node), i)
    }
}

fn keyword_speaker(keyword: &str) -> String {
    format!("`{}` is a keyword and can't be a character id", keyword)
}

fn option_text(token: &Token) -> Result<String, Diagnostic> {
    let (text, _) = scan_string(&token.payload).map_err(|e| Diagnostic::error(token.line, e))?;
    if text.trim().is_empty() {
        return Err(Diagnostic::error(token.line, "choice option text is empty"));
    }
    Ok(text)
}

fn words(payload: &str) -> Vec<&str> {
    payload.split_whitespace().collect()
}

fn parse_statement(token: &Token) -> Result<Node, Diagnostic> {
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

fn split_with(payload: &str, line: usize) -> Result<(Vec<&str>, Option<Transition>), Diagnostic> {
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

fn parse_transition(
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

fn parse_position(name: &str, line: usize) -> Result<Position, Diagnostic> {
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

fn position_names() -> String {
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

fn keyword_hint(payload: &str) -> String {
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

fn speaker_id(speaker: &str, line: usize) -> Result<String, Diagnostic> {
    if let Some(variable) = speaker
        .strip_prefix('{')
        .and_then(|rest| rest.strip_suffix('}'))
    {
        identifier(variable, "variable name", line)?;
        return Ok(speaker.to_string());
    }

    character_id(speaker, line)
}

fn character_id(id: &str, line: usize) -> Result<String, Diagnostic> {
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

fn parse_set(payload: &str, line: usize) -> Result<Node, Diagnostic> {
    let (var_id, literal) = payload
        .split_once('=')
        .ok_or_else(|| Diagnostic::error(line, "expected `set <variable> = <value>`"))?;

    Ok(Node::Set {
        var_id: identifier(var_id.trim(), "variable name", line)?,
        value: parse_value(literal.trim(), line)?,
    })
}

fn parse_add(payload: &str, line: usize) -> Result<Node, Diagnostic> {
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
