use vn_script::translate::{Catalog, Entry, StringKind, UI_BUCKET};

pub struct Line {
    pub context: String,
    pub text: String,
    pub fuzzy: bool,
}

pub fn context_of(bucket: &str, owner: &str, key: &str) -> String {
    match bucket {
        "story" => format!("story|{}|{}", owner, key),
        other => format!("{}|{}", other, key),
    }
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

fn quoted(keyword: &str, text: &str) -> String {
    if !text.contains('\n') {
        return format!("{} \"{}\"\n", keyword, escape(text));
    }

    let mut out = format!("{} \"\"\n", keyword);
    let mut rest = text;
    while let Some(at) = rest.find('\n') {
        out.push_str(&format!("\"{}\"\n", escape(&rest[..=at])));
        rest = &rest[at + 1..];
    }
    if !rest.is_empty() {
        out.push_str(&format!("\"{}\"\n", escape(rest)));
    }
    out
}

fn entry(out: &mut String, context: &str, entry: &Entry, place: Option<(&str, usize)>) {
    let what = match (entry.kind, &entry.speaker) {
        (StringKind::Dialogue, Some(speaker)) => format!("{} (dialogue)", speaker),
        (kind, _) => kind.name().to_string(),
    };
    out.push_str(&format!("#. {}\n", what));

    if let Some((file, line)) = place {
        match line {
            0 => out.push_str(&format!("#: {}\n", file)),
            line => out.push_str(&format!("#: {}:{}\n", file, line)),
        }
    }

    if entry.needs_review() {
        out.push_str("#, fuzzy\n");
    }

    let obsolete = entry.stale;
    if obsolete {
        out.push_str("#~ ");
    }
    out.push_str(&quoted("msgctxt", context));
    if obsolete {
        out.push_str("#~ ");
    }
    out.push_str(&quoted("msgid", &entry.source));
    if obsolete {
        out.push_str("#~ ");
    }
    out.push_str(&quoted("msgstr", &entry.text));
    out.push('\n');
}

pub fn export(catalog: &Catalog) -> String {
    let mut out = String::new();
    out.push_str("msgid \"\"\nmsgstr \"\"\n");
    out.push_str("\"Project-Id-Version: vn\\n\"\n");
    out.push_str(&format!("\"Language: {}\\n\"\n", catalog.language));
    out.push_str("\"MIME-Version: 1.0\\n\"\n");
    out.push_str("\"Content-Type: text/plain; charset=UTF-8\\n\"\n");
    out.push_str("\"Content-Transfer-Encoding: 8bit\\n\"\n");
    out.push('\n');

    for (file, entries) in &catalog.story {
        let mut ordered: Vec<(&String, &Entry)> = entries.iter().collect();
        ordered.sort_by_key(|(key, e)| (e.line, e.source.as_str(), key.as_str()));
        for (key, e) in ordered {
            entry(
                &mut out,
                &context_of("story", file, key),
                e,
                Some((file, e.line)),
            );
        }
    }

    for (id, e) in &catalog.names {
        entry(&mut out, &context_of("name", "", id), e, None);
    }

    let mut ui: Vec<(&String, &Entry)> = catalog.ui.iter().collect();
    ui.sort_by_key(|(key, e)| (e.source.as_str(), key.as_str()));
    for (key, e) in ui {
        entry(&mut out, &context_of(UI_BUCKET, "", key), e, None);
    }

    out
}

#[derive(Default)]
struct Pending {
    context: Option<String>,
    id: Option<String>,
    text: Option<String>,
    fuzzy: bool,
}

pub fn import(source: &str) -> Result<Vec<Line>, String> {
    let mut lines = Vec::new();
    let mut pending = Pending::default();
    let mut field: Option<&'static str> = None;

    let finish = |pending: &mut Pending, lines: &mut Vec<Line>| {
        let taken = std::mem::take(pending);
        let (Some(context), Some(id), Some(text)) = (taken.context, taken.id, taken.text) else {
            return;
        };
        if id.is_empty() && context.is_empty() {
            return;
        }
        lines.push(Line {
            context,
            text,
            fuzzy: taken.fuzzy,
        });
    };

    for (number, raw) in source.lines().enumerate() {
        let line = raw.trim();

        if line.is_empty() {
            finish(&mut pending, &mut lines);
            field = None;
            continue;
        }
        if let Some(rest) = line.strip_prefix('#') {
            if pending.text.is_some() {
                finish(&mut pending, &mut lines);
            }
            field = None;
            if rest.starts_with('~') {
                pending = Pending::default();
                continue;
            }
            if rest.starts_with(',') && rest.contains("fuzzy") {
                pending.fuzzy = true;
            }
            continue;
        }

        let (keyword, rest) = match line.split_once(' ') {
            Some((keyword, rest)) if keyword.starts_with("msg") => (Some(keyword), rest),
            _ => (None, line),
        };

        let value = unquote(rest.trim(), number + 1)?;

        match keyword {
            Some("msgctxt") => {
                if pending.text.is_some() {
                    finish(&mut pending, &mut lines);
                }
                pending.context = Some(value);
                field = Some("msgctxt");
            }
            Some("msgid") => {
                if pending.text.is_some() {
                    finish(&mut pending, &mut lines);
                }
                pending.id = Some(value);
                field = Some("msgid");
            }
            Some("msgstr") => {
                pending.text = Some(value);
                field = Some("msgstr");
            }
            Some(other) => return Err(format!("line {}: unknown keyword {}", number + 1, other)),
            None => match field {
                Some("msgctxt") => push(&mut pending.context, value),
                Some("msgid") => push(&mut pending.id, value),
                Some("msgstr") => push(&mut pending.text, value),
                _ => return Err(format!("line {}: a string with no keyword", number + 1)),
            },
        }
    }

    finish(&mut pending, &mut lines);
    Ok(lines)
}

fn push(field: &mut Option<String>, value: String) {
    match field {
        Some(existing) => existing.push_str(&value),
        None => *field = Some(value),
    }
}

fn unquote(text: &str, number: usize) -> Result<String, String> {
    let inner = text
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .ok_or_else(|| format!("line {}: expected a quoted string", number))?;

    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some(other) => return Err(format!("line {}: unknown escape \\{}", number, other)),
            None => return Err(format!("line {}: a string ends with a backslash", number)),
        }
    }
    Ok(out)
}

pub struct Applied {
    pub translated: usize,
    pub unreviewed: usize,
    pub unknown: Vec<String>,
}

pub fn apply(catalog: &mut Catalog, lines: Vec<Line>) -> Applied {
    let mut applied = Applied {
        translated: 0,
        unreviewed: 0,
        unknown: Vec::new(),
    };

    for line in lines {
        if line.text.is_empty() {
            continue;
        }
        let fuzzy = line.fuzzy;
        let parts: Vec<&str> = line.context.split('|').collect();
        let found = match parts.as_slice() {
            ["story", file, key] => catalog
                .story
                .get_mut(*file)
                .and_then(|entries| entries.get_mut(*key)),
            ["name", id] => catalog.names.get_mut(*id),
            [UI_BUCKET, key] => catalog.ui.get_mut(*key),
            _ => None,
        };

        match found {
            Some(entry) => {
                entry.text = line.text;
                entry.fuzzy = fuzzy;
                match fuzzy {
                    true => applied.unreviewed += 1,
                    false => applied.translated += 1,
                }
            }
            None => applied.unknown.push(line.context),
        }
    }

    applied
}
