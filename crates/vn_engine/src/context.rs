use std::rc::Rc;

use raylib::math::Rectangle;
use raylib::texture::Image;
use raylib::{RaylibHandle, RaylibThread};

use vn_script::StoryVm;

use crate::screens::{CONFIRM_OVERLAY, Confirm};
use crate::{
    Audio, Characters, Commands, Fonts, GameState, Hooks, LoadReport, LoadWarning, OverlayRequest,
    ResourceManager, Rollback, SaveError, Saves, ScreenState, Settings, SettingsStore, TextRequest,
    Toast,
};

pub struct GameContext<'a> {
    pub rl: &'a mut RaylibHandle,
    pub thread: &'a RaylibThread,
    pub resources: &'a mut ResourceManager,
    pub story: &'a mut StoryVm,
    pub state: &'a mut GameState,
    pub saves: &'a Saves,
    pub previous: Option<&'a ScreenState>,
    pub rollback: &'a mut Rollback,
    pub settings: &'a mut SettingsStore,
    pub(crate) commands: Rc<Commands>,
    pub(crate) hooks: Rc<Hooks>,
    pub(crate) overlay_requests: &'a mut Vec<OverlayRequest>,
    pub(crate) toast: &'a mut Option<Toast>,
    pub(crate) text_request: &'a mut Option<TextRequest>,
    pub(crate) confirm_request: &'a mut Option<Confirm>,
    pub(crate) thumbnail: Option<&'a Image>,
    pub(crate) autosave_request: &'a mut bool,
    pub(crate) audio: &'a mut Audio,
    pub(crate) tooltip: &'a mut Option<String>,
}

impl GameContext<'_> {
    pub fn open_overlay(&mut self, name: impl Into<String>) {
        self.overlay_requests
            .push(OverlayRequest::Open(name.into()));
    }

    pub fn close_overlay(&mut self) {
        self.overlay_requests.push(OverlayRequest::Close);
    }

    pub fn close_overlays(&mut self) {
        self.overlay_requests.push(OverlayRequest::CloseAll);
    }

    pub fn notify(&mut self, text: impl Into<String>) {
        *self.toast = Some(Toast::info(text));
    }

    pub fn notify_error(&mut self, text: impl Into<String>) {
        *self.toast = Some(Toast::error(text));
    }

    pub fn ask_text(&mut self, request: TextRequest) -> Option<ScreenState> {
        match self.story.variable_type(&request.variable) {
            Some(vn_script::VarType::String) | None => {
                *self.text_request = Some(request);
                Some(ScreenState::TextInput)
            }
            Some(other) => {
                eprintln!(
                    "⚠️ Text input for '{}': variable is {}, not string",
                    request.variable, other
                );
                None
            }
        }
    }

    pub fn confirm(&mut self, confirm: Confirm) {
        *self.confirm_request = Some(confirm);
        self.open_overlay(CONFIRM_OVERLAY);
    }

    pub fn take_confirm_request(&mut self) -> Option<Confirm> {
        self.confirm_request.take()
    }

    pub fn take_text_request(&mut self) -> Option<TextRequest> {
        self.text_request.take()
    }

    pub fn save(&mut self, slot: &str) -> Result<(), SaveError> {
        crate::saves::save_game(
            self.saves,
            slot,
            self.story,
            self.state,
            self.rollback,
            self.thumbnail,
        )
    }

    pub fn view(&self) -> GameView<'_> {
        GameView {
            story: self.story,
            state: self.state,
            saves: self.saves,
            settings: &self.settings.values,
        }
    }

    pub fn tooltip(&mut self, rect: Rectangle, text: impl Into<String>) {
        if crate::ui::is_hovered(self.rl, rect) {
            *self.tooltip = Some(text.into());
        }
    }

    pub fn play_sound(&mut self, id: &str) {
        self.audio.play_sound(id);
    }

    pub fn music(&self) -> Option<&str> {
        self.audio.music()
    }

    pub fn autosave(&mut self) {
        if self.saves.autosaves() {
            *self.autosave_request = true;
        }
    }

    pub fn load(&mut self, slot: &str) -> Result<LoadReport, SaveError> {
        let file = self.saves.read(slot)?;
        let report = crate::saves::apply(&file, self.story, self.state)?;

        let restarted = report
            .warnings
            .iter()
            .any(|w| matches!(w, LoadWarning::SceneRestarted { .. }));
        if restarted {
            self.rollback.clear();
        } else {
            self.rollback.restore_history(file.rollback, self.story);
        }

        for warning in &report.warnings {
            eprintln!("⚠️ Load '{}': {}", slot, warning);
        }
        Ok(report)
    }

    pub fn run_command(&mut self, name: &str, args: &[String]) -> Option<ScreenState> {
        let commands = Rc::clone(&self.commands);
        commands.run(name, self, args)
    }

    pub fn run_scene_hooks(&mut self, scene: &str) -> Option<ScreenState> {
        let hooks = Rc::clone(&self.hooks);
        hooks.scene_entered(self, scene)
    }

    pub fn run_choice_hooks(&mut self, index: usize, text: &str) -> Option<ScreenState> {
        let hooks = Rc::clone(&self.hooks);
        hooks.choice_made(self, index, text)
    }
}

pub struct GameView<'a> {
    pub story: &'a StoryVm,
    pub state: &'a GameState,
    pub saves: &'a Saves,
    pub settings: &'a Settings,
}

pub struct DrawContext<'a> {
    pub resources: &'a ResourceManager,
    pub story: &'a StoryVm,
    pub state: &'a GameState,
    pub saves: &'a Saves,
    pub characters: &'a Characters,
    pub settings: &'a Settings,
    pub interactive: bool,
}

impl DrawContext<'_> {
    pub fn pointer_over(&self, rl: &RaylibHandle, rect: Rectangle) -> bool {
        self.interactive && crate::ui::is_hovered(rl, rect)
    }

    pub fn fonts(&self) -> &Fonts {
        &self.resources.fonts
    }

    pub fn view(&self) -> GameView<'_> {
        GameView {
            story: self.story,
            state: self.state,
            saves: self.saves,
            settings: self.settings,
        }
    }
}
