use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Event, StoryVm, VmError};
use crate::{Instruction, Position, Value};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorySnapshot {
    pub scene: Option<String>,
    pub offset: usize,
    pub scene_fingerprint: u64,
    pub pending_choice: bool,
    pub current: Option<Event>,
    pub variables: BTreeMap<String, Value>,
    pub active_characters: BTreeMap<String, String>,
    #[serde(default)]
    pub positions: BTreeMap<String, Position>,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub music: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreOutcome {
    Exact,
    SceneRestarted { scene: String },
}

impl StoryVm {
    pub fn snapshot(&self) -> StorySnapshot {
        let scene_start = self
            .current_scene
            .as_deref()
            .and_then(|scene| self.program.scenes.get(scene).copied())
            .unwrap_or(0);

        StorySnapshot {
            scene: self.current_scene.clone(),
            offset: self.ip.saturating_sub(scene_start),
            scene_fingerprint: self
                .current_scene
                .as_deref()
                .and_then(|scene| self.program.scene_fingerprint(scene))
                .unwrap_or(0),
            pending_choice: self.pending_choice.is_some(),
            current: self.current.clone(),
            variables: self
                .variables
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            active_characters: self
                .active_characters
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            positions: self
                .positions
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
            background: self.background.clone(),
            music: self.music.clone(),
        }
    }

    pub fn check_restore(&self, snapshot: &StorySnapshot) -> Result<RestoreOutcome, VmError> {
        let Some(scene) = &snapshot.scene else {
            return Ok(RestoreOutcome::Exact);
        };

        let range = self
            .program
            .scene_range(scene)
            .ok_or_else(|| VmError::UnknownScene(scene.clone()))?;
        let ip = range.start + snapshot.offset;

        let unchanged = self.program.scene_fingerprint(scene) == Some(snapshot.scene_fingerprint)
            && ip < range.end
            && (!snapshot.pending_choice
                || matches!(self.program.instructions[ip], Instruction::Choice { .. }));

        Ok(if unchanged {
            RestoreOutcome::Exact
        } else {
            RestoreOutcome::SceneRestarted {
                scene: scene.clone(),
            }
        })
    }

    pub fn restore(&mut self, snapshot: &StorySnapshot) -> Result<RestoreOutcome, VmError> {
        let outcome = self.check_restore(snapshot)?;

        self.variables = self.schema.defaults().into_iter().collect();
        self.variables.extend(
            snapshot
                .variables
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );
        self.active_characters = snapshot.active_characters.clone().into_iter().collect();
        self.positions = snapshot.positions.clone().into_iter().collect();
        self.background = snapshot.background.clone();
        self.music = snapshot.music.clone();
        self.pending_transition = None;
        self.pending_choice = None;
        self.current = None;

        match (&snapshot.scene, &outcome) {
            (None, _) => self.reset_position(),
            (Some(scene), RestoreOutcome::SceneRestarted { .. }) => self.enter_scene(scene)?,
            (Some(scene), RestoreOutcome::Exact) => {
                self.enter_scene(scene)?;
                self.entered = false;
                self.ip += snapshot.offset;
                self.pending_choice = snapshot.pending_choice.then_some(self.ip);
                self.current = snapshot.current.clone();
            }
        }

        Ok(outcome)
    }
}
