use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use vn_script::{Catalog, StoryVm, Value, interpolate, referenced_variables};

pub const SEEN_FILE_NAME: &str = "seen.json";
pub const LOG_LIMIT: usize = 300;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayModes {
    pub auto: bool,
    pub skip: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spoken {
    #[serde(default)]
    pub file: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: BTreeMap<String, String>,
}

impl Spoken {
    pub fn capture(story: &StoryVm, source: &str) -> Self {
        Self::with_variables(
            story.current_file().unwrap_or_default(),
            source,
            story.variables(),
        )
    }

    pub fn with_variables(file: &str, source: &str, variables: &HashMap<String, Value>) -> Self {
        let fields = referenced_variables(source)
            .into_iter()
            .filter_map(|name| {
                let value = variables.get(name)?;
                Some((name.to_string(), value.to_string()))
            })
            .collect();
        Self {
            file: file.to_string(),
            source: source.to_string(),
            fields,
        }
    }

    pub fn text(&self, catalog: Option<&Catalog>) -> String {
        let text = catalog
            .and_then(|catalog| catalog.text(&self.file, &self.source))
            .unwrap_or(&self.source);
        let fields: HashMap<String, Value> = self
            .fields
            .iter()
            .map(|(name, value)| (name.clone(), Value::String(value.clone())))
            .collect();
        interpolate(text, &fields)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEntry {
    Line {
        speaker: Option<String>,
        #[serde(flatten)]
        said: Spoken,
    },
    Choice {
        #[serde(flatten)]
        said: Spoken,
    },
}

impl LogEntry {
    pub fn said(&self) -> &Spoken {
        match self {
            LogEntry::Line { said, .. } | LogEntry::Choice { said } => said,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionLog {
    entries: Vec<LogEntry>,
    visible: usize,
    limit: usize,
}

impl Default for SessionLog {
    fn default() -> Self {
        Self::new(LOG_LIMIT)
    }
}

impl SessionLog {
    pub fn new(limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            visible: 0,
            limit: limit.max(1),
        }
    }

    pub fn push(&mut self, entry: LogEntry) {
        self.entries.truncate(self.visible);
        self.entries.push(entry);
        if self.entries.len() > self.limit {
            let extra = self.entries.len() - self.limit;
            self.entries.drain(..extra);
        }
        self.visible = self.entries.len();
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries[..self.visible]
    }

    pub fn len(&self) -> usize {
        self.visible
    }

    pub fn is_empty(&self) -> bool {
        self.visible == 0
    }

    pub fn show(&mut self, len: usize) {
        self.visible = len.min(self.entries.len());
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.visible = 0;
    }

    pub fn replace(&mut self, entries: Vec<LogEntry>) {
        self.entries = entries;
        if self.entries.len() > self.limit {
            let extra = self.entries.len() - self.limit;
            self.entries.drain(..extra);
        }
        self.visible = self.entries.len();
    }
}

#[derive(Debug, Default)]
pub struct SeenLines {
    path: Option<PathBuf>,
    keys: BTreeSet<u64>,
    unsaved: usize,
}

impl SeenLines {
    pub fn in_memory() -> Self {
        Self::default()
    }

    pub fn load(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let keys = match fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_else(|e| {
                eprintln!("⚠️ {}: {}; starting with no seen lines", path.display(), e);
                BTreeSet::new()
            }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => BTreeSet::new(),
            Err(e) => {
                eprintln!("⚠️ {}: {}; starting with no seen lines", path.display(), e);
                BTreeSet::new()
            }
        };
        Self {
            path: Some(path),
            keys,
            unsaved: 0,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn contains(&self, key: u64) -> bool {
        self.keys.contains(&key)
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn insert(&mut self, key: u64) {
        if self.keys.insert(key) {
            self.unsaved += 1;
            if self.unsaved >= 25 {
                self.save();
            }
        }
    }

    pub fn save(&mut self) {
        let Some(path) = &self.path else {
            return;
        };
        if self.unsaved == 0 && path.exists() {
            return;
        }
        let json = serde_json::to_string(&self.keys).expect("seen lines serialize");
        match super::persistent::write_atomic(path, json.as_bytes()) {
            Ok(()) => self.unsaved = 0,
            Err(e) => eprintln!("⚠️ Could not save {}: {}", path.display(), e),
        }
    }
}
