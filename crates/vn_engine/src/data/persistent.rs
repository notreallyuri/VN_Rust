use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use super::state::GameState;

pub const PERSISTENT_FILE_NAME: &str = "persistent.json";

const VERSION: u32 = 1;
const SAVE_INTERVAL: f64 = 1.0;

#[derive(Serialize, Deserialize)]
struct File {
    version: u32,
    values: BTreeMap<String, Json>,
}

#[derive(Default)]
pub struct Persistent {
    path: Option<PathBuf>,
    state: GameState,
    stored: BTreeMap<String, Json>,
    dirty: bool,
    saved_at: Option<f64>,
}

impl Persistent {
    pub fn in_memory() -> Self {
        Self::default()
    }

    pub fn insert<T>(&mut self, initial: T)
    where
        T: Clone + Serialize + DeserializeOwned + 'static,
    {
        self.state.insert(initial);
    }

    pub fn load(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        let file = match fs::read_to_string(&path) {
            Ok(json) => match serde_json::from_str::<File>(&json) {
                Ok(file) => Some(file),
                Err(e) => {
                    eprintln!("⚠️ {}: {}; starting with defaults", path.display(), e);
                    backup(&path);
                    None
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => {
                eprintln!("⚠️ {}: {}; not writing it this session", path.display(), e);
                return;
            }
        };
        if let Some(file) = file {
            if file.version > VERSION {
                eprintln!(
                    "⚠️ {} is from a newer version ({}); reading it but not writing it",
                    path.display(),
                    file.version
                );
                self.state.restore(&file.values);
                return;
            }
            let failed = self.state.restore(&file.values);
            for error in &failed {
                eprintln!("⚠️ {}: {}; using its default", path.display(), error);
            }
            if !failed.is_empty() {
                backup(&path);
            }
            self.stored = file.values;
        }
        self.path = Some(path);
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn try_get<T: 'static>(&self) -> Option<&T> {
        self.state.try_get()
    }

    pub fn try_get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let value = self.state.try_get_mut();
        self.dirty |= value.is_some();
        value
    }

    pub fn get<T: 'static>(&self) -> &T {
        self.try_get().unwrap_or_else(|| {
            panic!(
                "{} is not registered with VnApp::persistent",
                std::any::type_name::<T>()
            )
        })
    }

    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        self.try_get_mut().unwrap_or_else(|| {
            panic!(
                "{} is not registered with VnApp::persistent",
                std::any::type_name::<T>()
            )
        })
    }

    pub fn save(&mut self) {
        if !std::mem::take(&mut self.dirty) {
            return;
        }
        let Some(path) = &self.path else {
            return;
        };
        let current = match self.state.to_json() {
            Ok(current) => current,
            Err(e) => {
                eprintln!("⚠️ Could not save {}: {}", path.display(), e);
                self.dirty = true;
                return;
            }
        };
        let mut values = self.stored.clone();
        values.extend(current);
        if values == self.stored && path.exists() {
            return;
        }
        let file = File {
            version: VERSION,
            values,
        };
        let json = serde_json::to_vec_pretty(&file).expect("persistent values serialize");
        match write_atomic(path, &json) {
            Ok(()) => self.stored = file.values,
            Err(e) => {
                eprintln!("⚠️ Could not save {}: {}", path.display(), e);
                self.dirty = true;
            }
        }
    }

    pub(crate) fn save_if_due(&mut self, now: f64) {
        if self.dirty && self.saved_at.is_none_or(|at| now - at >= SAVE_INTERVAL) {
            self.saved_at = Some(now);
            self.save();
        }
    }
}

fn backup(path: &Path) {
    let copy = path.with_extension("json.bak");
    if let Err(e) = fs::copy(path, &copy) {
        eprintln!("⚠️ Could not back up {}: {}", path.display(), e);
    } else {
        eprintln!("⚠️ Kept a copy at {}", copy.display());
    }
}

pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let temp = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&temp)?;
        io::Write::write_all(&mut file, bytes)?;
        file.sync_all()?;
    }
    fs::rename(&temp, path)
}
