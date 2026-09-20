use vn_script::StoryVm;

pub fn label<'a>(story: &'a StoryVm, text: &'a str) -> &'a str {
    story
        .catalog()
        .and_then(|catalog| catalog.ui_text(text))
        .unwrap_or(text)
}

pub fn fill(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('}').map(|i| start + i) else {
            rest = &rest[start..];
            break;
        };
        let name = &rest[start + 1..end];
        match fields.iter().find(|(key, _)| *key == name) {
            Some((_, value)) => out.push_str(value),
            None => out.push_str(&rest[start..=end]),
        }
        rest = &rest[end + 1..];
    }

    out.push_str(rest);
    out
}

pub const MESSAGES: &[&str] = &[
    "No saved game yet",
    "Quick saved",
    "Quick loaded",
    "Could not continue: {reason}",
    "Quick save failed: {reason}",
    "Quick load failed: {reason}",
    "Saved to {slot}",
    "Loaded {slot}",
    "Deleted {slot}",
    "Save failed: {reason}",
    "Load failed: {reason}",
    "Delete failed: {reason}",
    "Slot {number}",
    "Quick save",
    "Autosave",
    "Overwrite {slot}?",
    "Delete {slot}? This can't be undone.",
    "Screenshot saved",
    "Could not save the screenshot",
    "Story reloaded",
    "Story reloaded; scene '{scene}' restarted",
    "Story not reloaded: {reason}",
    "That language could not be loaded",
];
