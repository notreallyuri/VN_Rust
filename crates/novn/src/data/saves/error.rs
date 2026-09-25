use std::fmt;
use std::io;
use std::path::PathBuf;

use novn_script::VmError;

use crate::data::state::StateError;

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
    NewerGameVersion {
        found: u32,
        supported: u32,
    },
    Migration {
        path: PathBuf,
        from: u32,
        message: String,
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
            SaveError::NewerGameVersion { found, supported } => write!(
                f,
                "save version {} is newer than this game supports ({})",
                found, supported
            ),
            SaveError::Migration {
                path,
                from,
                message,
            } => write!(
                f,
                "{}: migrating from version {} failed: {}",
                path.display(),
                from,
                message
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
            SaveError::NewerFormat { .. } | SaveError::NewerGameVersion { .. } => {
                "This save was made by a newer version of the game.".to_string()
            }
            SaveError::Migration { .. } => {
                "This save couldn't be updated for this version of the game.".to_string()
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
