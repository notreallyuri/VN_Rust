use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use vn_script::{Event, RestoreOutcome, StorySnapshot, StoryVm};

use super::{LoadReport, LoadWarning, SaveError};
use crate::{Checkpoint, GameState, LogEntry};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveFile {
    pub format_version: u32,
    pub game: String,
    #[serde(default)]
    pub game_version: u32,
    pub saved_at: u64,
    pub summary: String,
    pub story: StorySnapshot,
    pub state: BTreeMap<String, Json>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rollback: Vec<Checkpoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub log: Vec<LogEntry>,
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

pub(super) fn summary(story: &StoryVm) -> String {
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
