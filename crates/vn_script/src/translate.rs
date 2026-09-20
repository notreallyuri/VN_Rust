use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{Instruction, Program, Schema};

pub const TRANSLATION_FORMAT_VERSION: u32 = 1;
pub const UI_BUCKET: &str = "ui";
pub const UI_STRINGS_FILE: &str = "ui.json";
pub const LANG_DIR: &str = "lang";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StringKind {
    #[default]
    Dialogue,
    Narration,
    Choice,
    Name,
    Ui,
}

impl StringKind {
    pub fn name(self) -> &'static str {
        match self {
            StringKind::Dialogue => "dialogue",
            StringKind::Narration => "narration",
            StringKind::Choice => "choice",
            StringKind::Name => "name",
            StringKind::Ui => "ui",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub file: String,
    pub line: usize,
    pub kind: StringKind,
    pub speaker: Option<String>,
    pub text: String,
}

pub fn file_key(name: &str) -> &str {
    name.rsplit(['/', '\\']).next().unwrap_or(name)
}

pub fn key(file: &str, source: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in file.bytes().chain([0x1f]).chain(source.bytes()) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x00000100000001b3);
    }
    format!("{:016x}", hash)
}

pub fn extract(program: &Program) -> Vec<Source> {
    let mut strings = Vec::new();
    for (index, instruction) in program.instructions.iter().enumerate() {
        let location = program.location(index);
        let file = program
            .file_name(location.file)
            .map(file_key)
            .unwrap_or_default()
            .to_string();

        match instruction {
            Instruction::Say { char_id, text } => strings.push(Source {
                file,
                line: location.line,
                kind: match char_id {
                    Some(_) => StringKind::Dialogue,
                    None => StringKind::Narration,
                },
                speaker: char_id.clone(),
                text: text.clone(),
            }),
            Instruction::Choice { options } => {
                for (text, _) in options {
                    strings.push(Source {
                        file: file.clone(),
                        line: location.line,
                        kind: StringKind::Choice,
                        speaker: None,
                        text: text.clone(),
                    });
                }
            }
            _ => {}
        }
    }
    strings
}

pub fn extract_names(schema: &Schema) -> Vec<(String, String)> {
    schema
        .characters
        .iter()
        .map(|(id, character)| (id.clone(), character.name.clone()))
        .collect()
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    #[serde(default)]
    pub kind: StringKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub line: usize,
    pub source: String,
    #[serde(default)]
    pub text: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub stale: bool,
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl Entry {
    pub fn translated(&self) -> Option<&str> {
        (!self.text.is_empty() && !self.stale).then_some(self.text.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalog {
    pub format_version: u32,
    pub language: String,
    #[serde(default)]
    pub story: BTreeMap<String, BTreeMap<String, Entry>>,
    #[serde(default)]
    pub names: BTreeMap<String, Entry>,
    #[serde(default)]
    pub ui: BTreeMap<String, Entry>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Refresh {
    pub added: usize,
    pub stale: usize,
    pub translated: usize,
    pub total: usize,
}

impl Refresh {
    pub fn missing(&self) -> usize {
        self.total - self.translated
    }
}

impl Catalog {
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            format_version: TRANSLATION_FORMAT_VERSION,
            language: language.into(),
            story: BTreeMap::new(),
            names: BTreeMap::new(),
            ui: BTreeMap::new(),
        }
    }

    pub fn text(&self, file: &str, source: &str) -> Option<&str> {
        let file = file_key(file);
        self.story.get(file)?.get(&key(file, source))?.translated()
    }

    pub fn name(&self, id: &str, source: &str) -> Option<&str> {
        let entry = self.names.get(id)?;
        (entry.source == source).then(|| entry.translated())?
    }

    pub fn ui_text(&self, source: &str) -> Option<&str> {
        self.ui.get(&key(UI_BUCKET, source))?.translated()
    }

    pub fn refresh_ui(&mut self, strings: &[String]) -> usize {
        let mut added = 0;
        let mut seen = Vec::with_capacity(strings.len());
        for source in strings {
            let key = key(UI_BUCKET, source);
            seen.push(key.clone());
            let entry = self.ui.entry(key).or_insert_with(|| {
                added += 1;
                Entry {
                    kind: StringKind::Ui,
                    source: source.clone(),
                    ..Entry::default()
                }
            });
            entry.kind = StringKind::Ui;
            entry.source = source.clone();
            entry.stale = false;
        }
        for (key, entry) in &mut self.ui {
            entry.stale = !seen.contains(key);
        }
        added
    }

    pub fn refresh(&mut self, strings: &[Source], names: &[(String, String)]) -> Refresh {
        let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut refresh = Refresh::default();

        for string in strings {
            let key = key(&string.file, &string.text);
            seen.entry(string.file.clone())
                .or_default()
                .push(key.clone());

            let file = self.story.entry(string.file.clone()).or_default();
            let entry = file.entry(key).or_insert_with(|| {
                refresh.added += 1;
                Entry {
                    source: string.text.clone(),
                    ..Entry::default()
                }
            });
            entry.kind = string.kind;
            entry.speaker = string.speaker.clone();
            entry.line = string.line;
            entry.source = string.text.clone();
            entry.stale = false;
        }

        for (file, entries) in &mut self.story {
            let live = seen.get(file);
            for (key, entry) in entries.iter_mut() {
                entry.stale = !live.is_some_and(|keys| keys.contains(key));
            }
        }

        for (id, source) in names {
            let entry = self.names.entry(id.clone()).or_insert_with(|| {
                refresh.added += 1;
                Entry {
                    kind: StringKind::Name,
                    source: source.clone(),
                    ..Entry::default()
                }
            });
            entry.kind = StringKind::Name;
            entry.stale = entry.source != *source;
        }
        let ids: Vec<&String> = names.iter().map(|(id, _)| id).collect();
        for (id, entry) in &mut self.names {
            if !ids.contains(&id) {
                entry.stale = true;
            }
        }

        refresh.total = self.entries().filter(|entry| !entry.stale).count();
        refresh.translated = self.entries().filter(|e| e.translated().is_some()).count();
        refresh.stale = self.entries().filter(|e| e.stale).count();
        refresh
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.story
            .values()
            .flat_map(|file| file.values())
            .chain(self.names.values())
            .chain(self.ui.values())
    }

    pub fn missing(&self) -> Vec<&Entry> {
        self.entries()
            .filter(|entry| !entry.stale && entry.text.is_empty())
            .collect()
    }

    pub fn stale(&self) -> Vec<&Entry> {
        self.entries().filter(|entry| entry.stale).collect()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        let catalog: Catalog = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if catalog.format_version > TRANSLATION_FORMAT_VERSION {
            return Err(format!(
                "translation format version {} is newer than this build understands ({})",
                catalog.format_version, TRANSLATION_FORMAT_VERSION
            ));
        }
        Ok(catalog)
    }

    pub fn read(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let json = fs::read_to_string(path)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", path.display(), e)))?;
        Self::from_json(&json).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {}", path.display(), e),
            )
        })
    }

    pub fn write(&self, path: impl AsRef<Path>) -> io::Result<bool> {
        let path = path.as_ref();
        let json = format!("{}\n", self.to_json());
        if fs::read_to_string(path).is_ok_and(|existing| existing == json) {
            return Ok(false);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, json)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", path.display(), e)))?;
        Ok(true)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStrings {
    #[serde(default)]
    pub format_version: u32,
    #[serde(default)]
    pub strings: Vec<String>,
}

impl UiStrings {
    pub fn new(strings: impl IntoIterator<Item = String>) -> Self {
        let mut strings: Vec<String> = strings.into_iter().filter(|s| !s.is_empty()).collect();
        strings.sort();
        strings.dedup();
        Self {
            format_version: TRANSLATION_FORMAT_VERSION,
            strings,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    pub fn read(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let json = fs::read_to_string(path)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", path.display(), e)))?;
        Self::from_json(&json).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {}", path.display(), e),
            )
        })
    }

    pub fn write(&self, path: impl AsRef<Path>) -> io::Result<bool> {
        let path = path.as_ref();
        let json = format!("{}\n", self.to_json());
        if fs::read_to_string(path).is_ok_and(|existing| existing == json) {
            return Ok(false);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, json)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", path.display(), e)))?;
        Ok(true)
    }
}
