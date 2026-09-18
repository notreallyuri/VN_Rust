use std::rc::Rc;

use raylib::{RaylibHandle, RaylibThread};

use vn_script::StoryVm;

use crate::screens::{CONFIRM_OVERLAY, Confirm};
use crate::{
    Characters, Commands, Fonts, GameState, LoadReport, OverlayRequest, ResourceManager, Rollback,
    SaveError, Saves, ScreenState, Settings, SettingsStore, TextRequest, Toast,
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
    pub(crate) overlay_requests: &'a mut Vec<OverlayRequest>,
    pub(crate) toast: &'a mut Option<Toast>,
    pub(crate) text_request: &'a mut Option<TextRequest>,
    pub(crate) confirm_request: &'a mut Option<Confirm>,
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
        self.saves.save(slot, self.story, self.state)
    }

    pub fn load(&mut self, slot: &str) -> Result<LoadReport, SaveError> {
        let report = self.saves.load(slot, self.story, self.state)?;
        self.rollback.clear();
        for warning in &report.warnings {
            eprintln!("⚠️ Load '{}': {}", slot, warning);
        }
        Ok(report)
    }

    pub fn run_command(&mut self, name: &str, args: &[String]) -> Option<ScreenState> {
        let commands = Rc::clone(&self.commands);
        commands.run(name, self, args)
    }
}

pub struct DrawContext<'a> {
    pub resources: &'a ResourceManager,
    pub story: &'a StoryVm,
    pub state: &'a GameState,
    pub saves: &'a Saves,
    pub characters: &'a Characters,
    pub settings: &'a Settings,
}

impl DrawContext<'_> {
    pub fn fonts(&self) -> &Fonts {
        &self.resources.fonts
    }
}
