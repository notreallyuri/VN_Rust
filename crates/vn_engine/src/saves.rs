use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use raylib::texture::Image;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use vn_script::{Event, RestoreOutcome, StorySnapshot, StoryVm, VmError};

use crate::{Checkpoint, GameState, Rollback, StateError};

pub const SAVE_FORMAT_VERSION: u32 = 1;
pub const QUICK_SLOT: &str = "quick";
pub const AUTO_SLOT: &str = "auto";
pub const THUMBNAIL_WIDTH: i32 = 320;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveFile {
    pub format_version: u32,
    pub game: String,
    pub saved_at: u64,
    pub summary: String,
    pub story: StorySnapshot,
    pub state: BTreeMap<String, Json>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rollback: Vec<Checkpoint>,
}

#[derive(Debug)]
pub enum SaveError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Empty {
        slot: String,
    },
    Corrupt {
        path: PathBuf,
        source: serde_json::Error,
    },
    NewerFormat {
        found: u32,
        supported: u32,
    },
    OtherGame {
        found: String,
        expected: String,
    },
    Story(VmError),
    State(StateError),
    InvalidSlot(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Io { path, source } => write!(f, "{}: {}", path.display(), source),
            SaveError::Empty { slot } => write!(f, "slot '{}' is empty", slot),
            SaveError::Corrupt { path, source } => {
                write!(f, "{} is not a valid save: {}", path.display(), source)
            }
            SaveError::NewerFormat { found, supported } => write!(
                f,
                "save format {} is newer than this game supports ({})",
                found, supported
            ),
            SaveError::OtherGame { found, expected } => {
                write!(f, "save belongs to '{}', not '{}'", found, expected)
            }
            SaveError::Story(e) => write!(f, "story: {}", e),
            SaveError::State(e) => write!(f, "{}", e),
            SaveError::InvalidSlot(slot) => write!(f, "invalid slot name '{}'", slot),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SaveError::Io { source, .. } => Some(source),
            SaveError::Corrupt { source, .. } => Some(source),
            SaveError::Story(e) => Some(e),
            SaveError::State(e) => Some(e),
            _ => None,
        }
    }
}

impl SaveError {
    pub fn player_message(&self) -> String {
        match self {
            SaveError::Io { source, .. } => {
                format!("Could not access the save file ({}).", source.kind())
            }
            SaveError::Empty { .. } => "This slot is empty.".to_string(),
            SaveError::Corrupt { .. } => {
                "This save file is damaged and can't be loaded.".to_string()
            }
            SaveError::NewerFormat { .. } => {
                "This save was made by a newer version of the game.".to_string()
            }
            SaveError::OtherGame { .. } => "This save belongs to a different game.".to_string(),
            SaveError::Story(_) => {
                "This save points to a part of the story that no longer exists.".to_string()
            }
            SaveError::State(_) => {
                "This save's game data doesn't match this version of the game.".to_string()
            }
            SaveError::InvalidSlot(_) => "Invalid save slot.".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadWarning {
    SceneRestarted { scene: String },
    MissingState { key: String },
    UnknownState { key: String },
}

impl fmt::Display for LoadWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadWarning::SceneRestarted { scene } => write!(
                f,
                "scene '{}' changed since this save; restarted it with the saved state",
                scene
            ),
            LoadWarning::MissingState { key } => {
                write!(f, "no saved '{}' state; using its initial value", key)
            }
            LoadWarning::UnknownState { key } => {
                write!(f, "saved '{}' state is no longer registered; ignored", key)
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct LoadReport {
    pub warnings: Vec<LoadWarning>,
}

#[derive(Debug)]
pub struct SlotInfo {
    pub slot: String,
    pub save: Result<SaveFile, SaveError>,
}

impl SlotInfo {
    pub fn is_empty(&self) -> bool {
        matches!(self.save, Err(SaveError::Empty { .. }))
    }
}

#[derive(Debug, Clone)]
pub struct Saves {
    dir: PathBuf,
    game: String,
    autosave: bool,
    generation: std::cell::Cell<u64>,
    any_cache: std::cell::Cell<Option<(u64, bool)>>,
}

impl Saves {
    pub fn new(dir: impl Into<PathBuf>, game: impl Into<String>) -> Self {
        Self {
            dir: dir.into(),
            game: game.into(),
            autosave: false,
            generation: std::cell::Cell::new(0),
            any_cache: std::cell::Cell::new(None),
        }
    }

    pub fn with_autosave(mut self, enabled: bool) -> Self {
        self.autosave = enabled;
        self
    }

    pub fn autosaves(&self) -> bool {
        self.autosave
    }

    pub fn generation(&self) -> u64 {
        self.generation.get()
    }

    fn bump(&self) {
        self.generation.set(self.generation.get() + 1);
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn path(&self, slot: &str) -> Result<PathBuf, SaveError> {
        let valid = !slot.is_empty()
            && slot
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');

        if !valid {
            return Err(SaveError::InvalidSlot(slot.to_string()));
        }

        Ok(self.dir.join(format!("{}.json", slot)))
    }

    pub fn thumbnail_path(&self, slot: &str) -> Result<PathBuf, SaveError> {
        Ok(self.path(slot)?.with_extension("png"))
    }

    pub fn write_thumbnail(&self, slot: &str, image: Option<&Image>) -> Result<(), SaveError> {
        let path = self.thumbnail_path(slot)?;
        let Some(image) = image else {
            return remove_if_present(&path);
        };

        let mut thumbnail = image.clone();
        let height = THUMBNAIL_WIDTH * image.height().max(1) / image.width().max(1);
        thumbnail.resize(THUMBNAIL_WIDTH, height.max(1));

        let temp = path.with_extension("tmp.png");
        thumbnail.export_image(&temp.to_string_lossy());
        if !temp.exists() {
            return Err(SaveError::Io {
                path: temp,
                source: io::Error::other("the image could not be written"),
            });
        }
        fs::rename(&temp, &path).map_err(|source| SaveError::Io { path, source })?;
        self.bump();
        Ok(())
    }

    pub fn capture(&self, story: &StoryVm, state: &GameState) -> Result<SaveFile, SaveError> {
        Ok(SaveFile {
            format_version: SAVE_FORMAT_VERSION,
            game: self.game.clone(),
            saved_at: now(),
            summary: summary(story),
            story: story.snapshot(),
            state: state.to_json().map_err(SaveError::State)?,
            rollback: Vec::new(),
        })
    }

    pub fn save(&self, slot: &str, story: &StoryVm, state: &GameState) -> Result<(), SaveError> {
        let file = self.capture(story, state)?;
        self.write(slot, &file)
    }

    pub fn write(&self, slot: &str, file: &SaveFile) -> Result<(), SaveError> {
        let path = self.path(slot)?;
        let io_error = |path: &Path| {
            let path = path.to_path_buf();
            move |source| SaveError::Io { path, source }
        };

        fs::create_dir_all(&self.dir).map_err(io_error(&self.dir))?;

        let json = serde_json::to_vec_pretty(file).expect("save files serialize");
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, json).map_err(io_error(&temp))?;
        fs::rename(&temp, &path).map_err(io_error(&path))?;
        self.bump();
        Ok(())
    }

    pub fn read(&self, slot: &str) -> Result<SaveFile, SaveError> {
        let path = self.path(slot)?;

        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(SaveError::Empty {
                    slot: slot.to_string(),
                });
            }
            Err(source) => return Err(SaveError::Io { path, source }),
        };

        let header: Json = serde_json::from_slice(&bytes).map_err(|source| SaveError::Corrupt {
            path: path.clone(),
            source,
        })?;

        let found = header
            .get("format_version")
            .and_then(Json::as_u64)
            .unwrap_or(0) as u32;
        if found > SAVE_FORMAT_VERSION {
            return Err(SaveError::NewerFormat {
                found,
                supported: SAVE_FORMAT_VERSION,
            });
        }

        let file: SaveFile =
            serde_json::from_value(header).map_err(|source| SaveError::Corrupt { path, source })?;

        if file.game != self.game {
            return Err(SaveError::OtherGame {
                found: file.game,
                expected: self.game.clone(),
            });
        }

        Ok(file)
    }

    pub fn load(
        &self,
        slot: &str,
        story: &mut StoryVm,
        state: &mut GameState,
    ) -> Result<LoadReport, SaveError> {
        let file = self.read(slot)?;
        apply(&file, story, state)
    }

    pub fn delete(&self, slot: &str) -> Result<(), SaveError> {
        remove_if_present(&self.path(slot)?)?;
        remove_if_present(&self.thumbnail_path(slot)?)?;
        self.bump();
        Ok(())
    }

    pub fn slot(&self, slot: &str) -> SlotInfo {
        SlotInfo {
            slot: slot.to_string(),
            save: self.read(slot),
        }
    }

    pub fn has_any(&self) -> bool {
        let generation = self.generation();
        if let Some((seen, any)) = self.any_cache.get()
            && seen == generation
        {
            return any;
        }
        let any = self.latest().is_some();
        self.any_cache.set(Some((generation, any)));
        any
    }

    pub fn latest(&self) -> Option<(String, SaveFile)> {
        let entries = fs::read_dir(&self.dir).ok()?;

        entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.extension()? != "json" {
                    return None;
                }
                let slot = path.file_stem()?.to_str()?.to_string();
                let file = self.read(&slot).ok()?;
                Some((slot, file))
            })
            .max_by_key(|(_, file)| file.saved_at)
    }
}

pub(crate) fn save_game(
    saves: &Saves,
    slot: &str,
    story: &StoryVm,
    state: &GameState,
    rollback: &Rollback,
    thumbnail: Option<&Image>,
) -> Result<(), SaveError> {
    let mut file = saves.capture(story, state)?;
    file.rollback = rollback.history();
    saves.write(slot, &file)?;
    if let Err(e) = saves.write_thumbnail(slot, thumbnail) {
        eprintln!("⚠️ Thumbnail for '{}' not saved: {}", slot, e);
    }
    Ok(())
}

pub(crate) fn autosave(
    saves: &Saves,
    story: &StoryVm,
    state: &GameState,
    rollback: &Rollback,
    thumbnail: Option<&Image>,
) {
    if !saves.autosaves() || matches!(story.current(), None | Some(Event::End)) {
        return;
    }
    if let Err(e) = save_game(saves, AUTO_SLOT, story, state, rollback, thumbnail) {
        eprintln!("⚠️ Autosave failed: {}", e);
    }
}

fn remove_if_present(path: &Path) -> Result<(), SaveError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(SaveError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

pub fn default_saves_dir(game: &str) -> PathBuf {
    match platform_data_dir() {
        Some(base) => base.join(slug(game)),
        None => PathBuf::from("saves"),
    }
}

fn platform_data_dir() -> Option<PathBuf> {
    let env_dir = |name: &str| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };

    if cfg!(windows) {
        env_dir("APPDATA")
    } else if cfg!(target_os = "macos") {
        env_dir("HOME").map(|home| home.join("Library/Application Support"))
    } else {
        env_dir("XDG_DATA_HOME").or_else(|| env_dir("HOME").map(|home| home.join(".local/share")))
    }
}

pub fn slug(title: &str) -> String {
    let mut slug = String::new();
    for c in title.chars() {
        if c.is_alphanumeric() {
            slug.extend(c.to_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('_') {
            slug.push('_');
        }
    }
    let slug = slug.trim_end_matches('_');
    if slug.is_empty() {
        "game".to_string()
    } else {
        slug.to_string()
    }
}

pub fn apply(
    file: &SaveFile,
    story: &mut StoryVm,
    state: &mut GameState,
) -> Result<LoadReport, SaveError> {
    story.check_restore(&file.story).map_err(SaveError::Story)?;
    let pending = state.prepare_load(&file.state).map_err(SaveError::State)?;

    let outcome = story.restore(&file.story).map_err(SaveError::Story)?;
    let state_report = state.apply(pending);

    let mut warnings = Vec::new();
    if let RestoreOutcome::SceneRestarted { scene } = outcome {
        warnings.push(LoadWarning::SceneRestarted { scene });
    }
    warnings.extend(
        state_report
            .missing
            .into_iter()
            .map(|key| LoadWarning::MissingState { key }),
    );
    warnings.extend(
        state_report
            .unknown
            .into_iter()
            .map(|key| LoadWarning::UnknownState { key }),
    );

    Ok(LoadReport { warnings })
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn time_ago(saved_at: u64, now: u64) -> String {
    let seconds = now.saturating_sub(saved_at);
    let plural =
        |n: u64, unit: &str| format!("{} {}{} ago", n, unit, if n == 1 { "" } else { "s" });

    match seconds {
        0..=59 => "just now".to_string(),
        60..=3599 => plural(seconds / 60, "minute"),
        3600..=86_399 => plural(seconds / 3600, "hour"),
        86_400..=2_591_999 => plural(seconds / 86_400, "day"),
        _ => plural(seconds / 2_592_000, "month"),
    }
}

fn summary(story: &StoryVm) -> String {
    let line = match story.current() {
        Some(Event::Say {
            speaker: Some(speaker),
            text,
        }) => format!("{}: {}", speaker, text),
        Some(Event::Say {
            speaker: None,
            text,
        }) => text.clone(),
        Some(Event::Choice { .. }) => "Making a choice".to_string(),
        Some(Event::End) => "The end".to_string(),
        _ => String::new(),
    };

    const MAX: usize = 80;
    if line.chars().count() > MAX {
        let cut: String = line.chars().take(MAX - 1).collect();
        format!("{}…", cut.trim_end())
    } else {
        line
    }
}
