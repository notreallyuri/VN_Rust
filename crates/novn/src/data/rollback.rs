use std::collections::{BTreeMap, BTreeSet, VecDeque};

use novn_script::{RestoreOutcome, StorySnapshot, StoryVm};
use raylib::consts::KeyboardKey;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::data::state::GameState;

#[derive(Clone, Debug)]
pub struct RollbackConfig {
    pub enabled: bool,
    pub max_steps: usize,
    pub through_choices: bool,
    pub blocked_commands: BTreeSet<String>,
    pub save_history: bool,
    pub mouse_wheel: bool,
    pub back_keys: Vec<KeyboardKey>,
    pub forward_keys: Vec<KeyboardKey>,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_steps: 100,
            through_choices: true,
            blocked_commands: BTreeSet::new(),
            save_history: true,
            mouse_wheel: true,
            back_keys: vec![KeyboardKey::KEY_PAGE_UP],
            forward_keys: vec![KeyboardKey::KEY_PAGE_DOWN],
        }
    }
}

impl RollbackConfig {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn max_steps(mut self, steps: usize) -> Self {
        self.max_steps = steps.max(1);
        self
    }

    pub fn through_choices(mut self, allow: bool) -> Self {
        self.through_choices = allow;
        self
    }

    pub fn block_command(mut self, name: impl Into<String>) -> Self {
        self.blocked_commands.insert(name.into());
        self
    }

    pub fn save_history(mut self, save: bool) -> Self {
        self.save_history = save;
        self
    }

    pub fn mouse_wheel(mut self, enabled: bool) -> Self {
        self.mouse_wheel = enabled;
        self
    }

    pub fn back_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.back_keys = keys.into_iter().collect();
        self
    }

    pub fn forward_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.forward_keys = keys.into_iter().collect();
        self
    }
}

pub type VisualParameters = BTreeMap<String, BTreeMap<String, f32>>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub story: StorySnapshot,
    pub state: BTreeMap<String, Json>,
    pub barrier: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_len: Option<usize>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub visuals: VisualParameters,
}

#[derive(Debug, Default)]
pub struct Rollback {
    config: RollbackConfig,
    history: VecDeque<Checkpoint>,
    future: Vec<Checkpoint>,
    pending_barrier: bool,
}

impl Rollback {
    pub fn new(config: RollbackConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn config(&self) -> &RollbackConfig {
        &self.config
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.future.clear();
        self.pending_barrier = false;
    }

    pub fn history(&self) -> Vec<Checkpoint> {
        if self.config.enabled && self.config.save_history {
            self.history.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn restore_history(&mut self, checkpoints: Vec<Checkpoint>, story: &StoryVm) -> usize {
        self.clear();
        if !self.config.enabled {
            return 0;
        }

        let exact = |checkpoint: &Checkpoint| {
            matches!(
                story.check_restore(&checkpoint.story),
                Ok(RestoreOutcome::Exact)
            )
        };
        let usable = checkpoints.iter().rev().take_while(|c| exact(c)).count();
        let start = checkpoints.len() - usable.min(self.config.max_steps + 1);

        self.history = checkpoints.into_iter().skip(start).collect();
        self.history.len()
    }

    pub fn retain_valid(&mut self, story: &StoryVm) {
        let history: Vec<Checkpoint> = self.history.drain(..).collect();
        self.restore_history(history, story);
    }

    pub fn mark_barrier(&mut self) {
        self.pending_barrier = true;
    }

    pub fn blocks_command(&self, name: &str) -> bool {
        self.config.blocked_commands.contains(name)
    }

    pub fn record(&mut self, story: &StoryVm, state: &GameState) {
        self.record_with_log(story, state, None, VisualParameters::new());
    }

    pub fn visuals(&self) -> &VisualParameters {
        static NONE: std::sync::OnceLock<VisualParameters> = std::sync::OnceLock::new();
        self.history
            .back()
            .map(|checkpoint| &checkpoint.visuals)
            .unwrap_or_else(|| NONE.get_or_init(VisualParameters::new))
    }

    pub fn log_len(&self) -> Option<usize> {
        self.history
            .back()
            .and_then(|checkpoint| checkpoint.log_len)
    }

    pub fn record_with_log(
        &mut self,
        story: &StoryVm,
        state: &GameState,
        log_len: Option<usize>,
        visuals: VisualParameters,
    ) {
        if !self.config.enabled {
            return;
        }

        let snapshot = story.snapshot();
        let barrier = std::mem::take(&mut self.pending_barrier);

        if !barrier
            && self
                .history
                .back()
                .is_some_and(|last| last.story == snapshot && last.visuals == visuals)
        {
            return;
        }

        let state = match state.to_json() {
            Ok(state) => state,
            Err(e) => {
                eprintln!("⚠️ Rollback disabled for this step: {}", e);
                self.history.clear();
                return;
            }
        };

        if barrier {
            self.history.clear();
        }

        self.future.clear();
        self.history.push_back(Checkpoint {
            story: snapshot,
            state,
            barrier,
            log_len,
            visuals,
        });

        while self.history.len() > self.config.max_steps + 1 {
            self.history.pop_front();
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.config.enabled
            && self.history.len() >= 2
            && self.history.back().is_some_and(|last| !last.barrier)
    }

    pub fn can_go_forward(&self) -> bool {
        self.config.enabled && !self.future.is_empty()
    }

    pub fn steps_back(&self) -> usize {
        if !self.config.enabled {
            return 0;
        }
        let since_barrier = self
            .history
            .iter()
            .rev()
            .position(|checkpoint| checkpoint.barrier)
            .map_or(self.history.len(), |i| i + 1);
        since_barrier.saturating_sub(1)
    }

    pub fn back(&mut self, story: &mut StoryVm, state: &mut GameState) -> bool {
        if !self.can_go_back() {
            return false;
        }

        let current = self.history.pop_back().expect("checked above");
        self.future.push(current);
        self.restore_last(story, state)
    }

    pub fn forward(&mut self, story: &mut StoryVm, state: &mut GameState) -> bool {
        let Some(next) = self.future.pop() else {
            return false;
        };

        self.history.push_back(next);
        self.restore_last(story, state)
    }

    fn restore_last(&mut self, story: &mut StoryVm, state: &mut GameState) -> bool {
        let Some(checkpoint) = self.history.back() else {
            return false;
        };

        if let Err(e) = story.restore(&checkpoint.story) {
            eprintln!("⚠️ Rollback failed: {}", e);
            self.clear();
            return false;
        }

        match state.prepare_load(&checkpoint.state) {
            Ok(pending) => {
                state.apply(pending);
                true
            }
            Err(e) => {
                eprintln!("⚠️ Rollback failed: {}", e);
                self.clear();
                false
            }
        }
    }
}
