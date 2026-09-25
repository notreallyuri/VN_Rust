use novn_script::{Position, Schema, Transition, TransitionKind, VarType, keywords};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Keyword,
    Scene,
    Character,
    Image,
    Variable,
    Value,
    Command,
    Operator,
    Asset,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub label: String,
    pub kind: ItemKind,
    pub detail: Option<String>,
}

#[derive(Default)]
pub struct Context<'a> {
    pub schema: Option<&'a Schema>,
    pub scenes: Vec<String>,
    pub backgrounds: Vec<String>,
    pub choices: Vec<String>,
    pub previews: Vec<String>,
    pub music: Vec<String>,
    pub sounds: Vec<String>,
    pub voices: Vec<String>,
}

fn items<S: Into<String>>(labels: impl IntoIterator<Item = S>, kind: ItemKind) -> Vec<Item> {
    labels
        .into_iter()
        .map(|label| Item {
            label: label.into(),
            kind,
            detail: None,
        })
        .collect()
}

impl Context<'_> {
    fn characters(&self) -> Vec<Item> {
        let Some(schema) = self.schema else {
            return Vec::new();
        };
        schema
            .characters
            .iter()
            .map(|(id, def)| Item {
                label: id.clone(),
                kind: ItemKind::Character,
                detail: Some(def.name.clone()),
            })
            .collect()
    }

    fn images(&self, character: &str) -> Vec<Item> {
        let images = self
            .schema
            .and_then(|schema| schema.characters.get(character))
            .map(|def| def.images.clone())
            .unwrap_or_default();
        items(images, ItemKind::Image)
    }

    fn variables(&self, only: Option<fn(&VarType) -> bool>) -> Vec<Item> {
        let Some(schema) = self.schema else {
            return Vec::new();
        };
        schema
            .variables
            .iter()
            .filter(|(_, def)| only.is_none_or(|keep| keep(&def.ty)))
            .map(|(id, def)| Item {
                label: id.clone(),
                kind: ItemKind::Variable,
                detail: Some(def.ty.to_string()),
            })
            .collect()
    }

    fn is_variable(&self, word: &str) -> bool {
        self.schema
            .is_some_and(|schema| schema.variables.contains_key(word))
    }

    fn values(&self, variable: &str) -> Vec<Item> {
        let ty = self
            .schema
            .and_then(|schema| schema.variables.get(variable))
            .map(|def| &def.ty);
        match ty {
            Some(VarType::Bool) => items(["true", "false"], ItemKind::Value),
            Some(VarType::Enum(members)) => items(members.clone(), ItemKind::Value),
            _ => Vec::new(),
        }
    }

    fn command_words(&self, command: &str, at: usize) -> Vec<Item> {
        let Some(sig) = self.schema.and_then(|schema| schema.commands.get(command)) else {
            return Vec::new();
        };
        let kind = sig
            .required
            .iter()
            .chain(&sig.optional)
            .nth(at)
            .or(sig.rest.as_ref());

        match kind {
            Some(novn_script::ParamKind::Choice(members)) => {
                items(members.clone(), ItemKind::Value)
            }
            Some(novn_script::ParamKind::Bool) => items(["true", "false"], ItemKind::Value),
            _ => Vec::new(),
        }
    }

    fn commands(&self) -> Vec<Item> {
        let Some(schema) = self.schema else {
            return Vec::new();
        };
        schema
            .commands
            .iter()
            .map(|(name, sig)| Item {
                label: name.clone(),
                kind: ItemKind::Command,
                detail: Some(sig.usage(name)),
            })
            .collect()
    }
}

fn positions() -> Vec<Item> {
    items(
        Position::ALL.iter().map(ToString::to_string),
        ItemKind::Keyword,
    )
}

fn transitions() -> Vec<Item> {
    items(
        TransitionKind::ALL
            .iter()
            .map(|&kind| Transition { kind, millis: None }.to_string()),
        ItemKind::Keyword,
    )
}

fn in_string(prefix: &str) -> bool {
    let mut inside = false;
    let mut escaped = false;
    for c in prefix.chars() {
        match c {
            '\\' if inside && !escaped => escaped = true,
            '"' if !escaped => inside = !inside,
            _ => escaped = false,
        }
    }
    inside
}

pub fn complete(prefix: &str, ctx: &Context) -> Vec<Item> {
    if in_string(prefix) {
        let open = prefix.rfind('{');
        let close = prefix.rfind('}');
        return match (open, close) {
            (Some(open), close) if close.is_none_or(|close| close < open) => ctx.variables(None),
            _ => Vec::new(),
        };
    }

    if let Some(rest) = after_option_text(prefix) {
        return option_modifiers(rest, ctx);
    }

    let words: Vec<&str> = prefix.split_whitespace().collect();
    let typing = !prefix.ends_with(char::is_whitespace) && !words.is_empty();
    let before: &[&str] = if typing {
        &words[..words.len() - 1]
    } else {
        &words
    };

    let with = || items(["with"], ItemKind::Keyword);
    match before {
        [] => {
            let mut all = items(keywords(), ItemKind::Keyword);
            all.extend(ctx.characters());
            all
        }
        ["jump"] => items(ctx.scenes.clone(), ItemKind::Scene),
        ["show"] | ["remove"] => ctx.characters(),
        ["show", character] => ctx.images(character),
        ["show", _, _] => items(["at", "with"], ItemKind::Keyword),
        [.., "at"] => positions(),
        ["show", _, _, "at", _] => with(),
        [.., "with"] => transitions(),
        ["remove", _] | ["clear"] => with(),
        ["background"] => {
            let mut all = items(ctx.backgrounds.clone(), ItemKind::Asset);
            all.extend(items(["none"], ItemKind::Keyword));
            all
        }
        ["background", _] => with(),
        ["music"] => {
            let mut all = items(ctx.music.clone(), ItemKind::Asset);
            all.extend(items(["none"], ItemKind::Keyword));
            all
        }
        ["sound"] => items(ctx.sounds.clone(), ItemKind::Asset),
        ["voice"] => items(ctx.voices.clone(), ItemKind::Asset),
        ["set"] => ctx.variables(None),
        ["set", _] => items(["="], ItemKind::Operator),
        ["set", variable, "="] => ctx.values(variable),
        ["add"] => ctx.variables(Some(|ty| *ty == VarType::Int)),
        ["add", _] => items(["+=", "-="], ItemKind::Operator),
        ["call"] => ctx.commands(),
        ["call", command, args @ ..] => ctx.command_words(command, args.len()),
        ["choice"] => items(["final:"], ItemKind::Keyword),
        ["scene", _] => items(["nvl:", "adv:"], ItemKind::Keyword),
        ["if", ..] => {
            let last = before.last().copied().unwrap_or_default();
            let second = before.len().checked_sub(2).map(|i| before[i]);
            match last {
                "if" | "&&" | "||" => ctx.variables(None),
                word if ctx.is_variable(word) => {
                    items(["==", "!=", ">=", "<=", ">", "<"], ItemKind::Operator)
                }
                "==" | "!=" => second.map(|v| ctx.values(v)).unwrap_or_default(),
                _ => items(["&&", "||"], ItemKind::Operator),
            }
        }
        _ => Vec::new(),
    }
}

fn after_option_text(prefix: &str) -> Option<&str> {
    let trimmed = prefix.trim_start();
    if !trimmed.starts_with('"') {
        return None;
    }
    let (_, used) = novn_script::scan_string(trimmed).ok()?;
    Some(&trimmed[used..])
}

fn option_modifiers(rest: &str, ctx: &Context) -> Vec<Item> {
    let words: Vec<&str> = rest.split_whitespace().collect();
    let typing = !rest.ends_with(char::is_whitespace) && !words.is_empty();
    let before: &[&str] = if typing {
        &words[..words.len() - 1]
    } else {
        &words
    };

    match before {
        [.., "image"] => items(ctx.choices.clone(), ItemKind::Asset),
        [.., "preview"] => items(ctx.previews.clone(), ItemKind::Asset),
        [] => items(["when", "unless", "image", "preview"], ItemKind::Keyword),
        [.., last] if ctx.is_variable(last) => {
            items(["==", "!=", ">=", "<=", ">", "<"], ItemKind::Operator)
        }
        ["when"] | ["unless"] | [.., "&&"] | [.., "||"] => ctx.variables(None),
        [.., "==" | "!="] => match before.len().checked_sub(2).map(|i| before[i]) {
            Some(variable) => ctx.values(variable),
            None => Vec::new(),
        },
        _ => items(["image", "preview"], ItemKind::Keyword),
    }
}

pub fn word_at(line: &str, column: usize) -> Option<(usize, usize, &str)> {
    let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let at = column.min(chars.len());
    let mut start = at;
    while start > 0 && is_word(chars[start - 1].1) {
        start -= 1;
    }
    let mut end = at;
    while end < chars.len() && is_word(chars[end].1) {
        end += 1;
    }
    if start == end {
        return None;
    }
    let byte = |i: usize| chars.get(i).map_or(line.len(), |&(b, _)| b);
    Some((start, end, &line[byte(start)..byte(end)]))
}

pub fn hover(word: &str, ctx: &Context, scene: Option<(&str, usize)>) -> Option<String> {
    if let Some((file, line)) = scene {
        return Some(format!("**scene** `{}`\n\n{}:{}", word, file, line));
    }
    let schema = ctx.schema?;
    if let Some(def) = schema.characters.get(word) {
        let images = if def.images.is_empty() {
            "no images".to_string()
        } else {
            def.images.join(", ")
        };
        return Some(format!(
            "**{}** (character `{}`)\n\nImages: {}",
            def.name, word, images
        ));
    }
    if let Some(def) = schema.variables.get(word) {
        return Some(format!(
            "**variable** `{}`: {} = `{}`",
            word,
            def.ty,
            def.default.literal()
        ));
    }
    if let Some(sig) = schema.commands.get(word) {
        return Some(format!("**command**\n\n`{}`", sig.usage(word)));
    }
    None
}

pub fn scene_lines(text: &str) -> Vec<(String, usize)> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let rest = line.strip_prefix("scene ")?;
            let name = rest.trim_end().strip_suffix(':')?.trim();
            (!name.is_empty()).then(|| (name.to_string(), index))
        })
        .collect()
}
