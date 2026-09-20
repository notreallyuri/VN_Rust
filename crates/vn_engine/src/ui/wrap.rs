const NO_START: &str = concat!(
    "、。，．・：；？！ー〜ヽヾゝゞ々",
    "ぁぃぅぇぉっゃゅょゎァィゥェォッャュョヮヵヶ",
    "」』）〕］｝〉》」』】｣",
    "%‰°℃¢／゛゜ﾞﾟ"
);

const NO_END: &str = "「『（〔［｛〈《【｢＄£＃￥￦";

pub fn is_wide(c: char) -> bool {
    matches!(c as u32,
        0x1100..=0x115F
        | 0x2E80..=0x303E
        | 0x3041..=0x33FF
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xA000..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1F64F
        | 0x20000..=0x2FA1F
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit {
    pub text: String,
    pub space_before: bool,
}

pub fn units(paragraph: &str) -> Vec<Unit> {
    let mut units: Vec<Unit> = Vec::new();
    let mut word = String::new();
    let mut space = false;

    let flush = |word: &mut String, space: &mut bool, units: &mut Vec<Unit>| {
        if !word.is_empty() {
            units.push(Unit {
                text: std::mem::take(word),
                space_before: std::mem::take(space),
            });
        }
    };

    for c in paragraph.chars() {
        if c.is_whitespace() {
            flush(&mut word, &mut space, &mut units);
            space = true;
            continue;
        }

        if is_wide(c) {
            flush(&mut word, &mut space, &mut units);
            units.push(Unit {
                text: c.to_string(),
                space_before: std::mem::take(&mut space),
            });
            continue;
        }

        word.push(c);
    }
    flush(&mut word, &mut space, &mut units);

    glue(units)
}

fn glue(units: Vec<Unit>) -> Vec<Unit> {
    let mut out: Vec<Unit> = Vec::with_capacity(units.len());

    for unit in units {
        let joins = !unit.space_before
            && out.last().is_some_and(|last: &Unit| {
                last.text.ends_with(|c| NO_END.contains(c))
                    || unit.text.starts_with(|c| NO_START.contains(c))
            });
        match joins {
            true => out
                .last_mut()
                .expect("checked above")
                .text
                .push_str(&unit.text),
            false => out.push(unit),
        }
    }

    out
}

pub fn lines(paragraph: &str, fits: impl Fn(&str) -> bool) -> Vec<String> {
    let units = units(paragraph);
    if units.is_empty() {
        return vec![String::new()];
    }

    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();

    for unit in units {
        if line.is_empty() {
            line.push_str(&unit.text);
            continue;
        }
        let candidate = match unit.space_before {
            true => format!("{} {}", line, unit.text),
            false => format!("{}{}", line, unit.text),
        };
        if fits(&candidate) {
            line = candidate;
        } else {
            lines.push(std::mem::replace(&mut line, unit.text.clone()));
        }
    }
    lines.push(line);

    lines
}

pub fn breaks(text: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut at = 0;

    for unit in units(text) {
        if at > 0 {
            offsets.push(at);
        }
        at += unit.text.chars().count();
    }

    offsets
}
