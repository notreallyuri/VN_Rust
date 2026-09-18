use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub fullscreen: bool,
    pub text_speed: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            text_speed: 40,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_else(|e| {
                eprintln!("⚠️ {}: {}; using default settings", path.display(), e);
                Self::default()
            }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(e) => {
                eprintln!("⚠️ {}: {}; using default settings", path.display(), e);
                Self::default()
            }
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut json = serde_json::to_string_pretty(self).expect("settings serialize");
        json.push('\n');
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, json)?;
        fs::rename(&temp, path)
    }
}

#[derive(Debug)]
pub struct SettingsStore {
    pub values: Settings,
    path: PathBuf,
}

impl SettingsStore {
    pub fn load(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self {
            values: Settings::load(&path),
            path,
        }
    }

    pub fn in_memory() -> Self {
        Self {
            values: Settings::default(),
            path: PathBuf::new(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn update(&mut self, change: impl FnOnce(&mut Settings)) {
        let before = self.values.clone();
        change(&mut self.values);
        if self.values == before || self.path.as_os_str().is_empty() {
            return;
        }
        if let Err(e) = self.values.save(&self.path) {
            eprintln!(
                "⚠️ Could not save settings to {}: {}",
                self.path.display(),
                e
            );
        }
    }
}
