use std::collections::BTreeMap;

use novn_script::{Event, RestoreOutcome, StorySnapshot, StoryVm};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use super::{LoadReport, LoadWarning, SaveError};
use crate::data::rollback::{Checkpoint, VisualParameters};
use crate::data::session::{LogEntry, Spoken};
use crate::data::state::GameState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveFile {
    pub format_version: u32,
    pub game: String,
    #[serde(default)]
    pub game_version: u32,
    pub saved_at: u64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary: String,
    #[serde(default)]
    pub point: SavePoint,
    pub story: StorySnapshot,
    pub state: BTreeMap<String, Json>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rollback: Vec<Checkpoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub log: Vec<LogEntry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub visuals: VisualParameters,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SavePoint {
    Line {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        speaker: Option<String>,
        #[serde(flatten)]
        said: Spoken,
    },
    Choosing,
    End,
    #[default]
    Unknown,
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

pub(super) fn point(story: &StoryVm) -> SavePoint {
    match story.current() {
        Some(Event::Say { speaker, .. }) => match story.current_say() {
            Some((_, source)) => SavePoint::Line {
                speaker: speaker.clone(),
                said: Spoken::capture(story, source),
            },
            None => SavePoint::Unknown,
        },
        Some(Event::Choice { .. }) => SavePoint::Choosing,
        Some(Event::End) => SavePoint::End,
        _ => SavePoint::Unknown,
    }
}
