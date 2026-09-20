use std::fs;
use std::path::Path;

use vn_script::translate::{self, Catalog, Entry};
use vn_script::{LANG_DIR, Program, Schema, UI_STRINGS_FILE, UiStrings};

pub struct Status {
    pub language: String,
    pub total: usize,
    pub translated: usize,
    pub missing: Vec<Line>,
    pub stale: Vec<Line>,
}

pub struct Line {
    pub kind: &'static str,
    pub file: String,
    pub line: usize,
    pub source: String,
}

impl Line {
    pub fn place(&self) -> String {
        match self.line {
            0 => self.file.clone(),
            line => format!("{}:{}", self.file, line),
        }
    }
}

impl Status {
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty() && self.stale.is_empty()
    }
}

pub fn codes(root: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(root.join(LANG_DIR)) else {
        return Vec::new();
    };
    let mut codes: Vec<String> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .filter(|path| path.file_name().is_some_and(|name| name != UI_STRINGS_FILE))
        .filter_map(|path| path.file_stem()?.to_str().map(str::to_string))
        .collect();
    codes.sort();
    codes
}

pub fn status(
    root: &Path,
    code: &str,
    program: &Program,
    schema: Option<&Schema>,
) -> Result<Status, String> {
    let path = root.join(LANG_DIR).join(format!("{}.json", code));
    let mut catalog = Catalog::read(&path).map_err(|e| e.to_string())?;

    let ui = root.join(LANG_DIR).join(UI_STRINGS_FILE);
    if let Ok(strings) = UiStrings::read(&ui) {
        catalog.refresh_ui(&strings.strings);
    }
    let names = schema.map(translate::extract_names).unwrap_or_default();
    catalog.refresh(&translate::extract(program), &names);

    let mut status = Status {
        language: catalog.language.clone(),
        total: 0,
        translated: 0,
        missing: Vec::new(),
        stale: Vec::new(),
    };

    let mut sort = |entry: &Entry, file: String| {
        let line = || Line {
            kind: entry.kind.name(),
            file: file.clone(),
            line: entry.line,
            source: entry.source.clone(),
        };
        if entry.stale {
            status.stale.push(line());
            return;
        }
        status.total += 1;
        match entry.translated() {
            Some(_) => status.translated += 1,
            None => status.missing.push(line()),
        }
    };

    for (file, entries) in &catalog.story {
        for entry in entries.values() {
            sort(entry, file.clone());
        }
    }
    for (id, entry) in &catalog.names {
        sort(entry, format!("name {}", id));
    }
    for entry in catalog.ui.values() {
        sort(entry, "screens".to_string());
    }

    for lines in [&mut status.missing, &mut status.stale] {
        lines.sort_by(|a, b| {
            a.file
                .cmp(&b.file)
                .then(a.line.cmp(&b.line))
                .then(a.source.cmp(&b.source))
        });
    }

    Ok(status)
}
