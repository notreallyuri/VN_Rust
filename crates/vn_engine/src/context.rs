use std::rc::Rc;

use raylib::math::Rectangle;
use raylib::texture::Image;
use raylib::{RaylibHandle, RaylibThread};

use vn_script::StoryVm;

use crate::data::resources::ResourceManager;
use crate::data::rollback::Rollback;
use crate::data::saves::{LoadReport, LoadWarning, SaveError, Saves};
use crate::data::session::{PlayModes, SeenLines, SessionLog};
use crate::data::settings::{Settings, SettingsStore};
use crate::data::state::GameState;
use crate::game::audio::Audio;
use crate::game::characters::Characters;
use crate::game::commands::Commands;
use crate::game::hooks::Hooks;
use crate::input::navigation::{Focus, NavInput};
use crate::overlay::OverlayRequest;
use crate::screen::ScreenState;
use crate::screens::confirm::{CONFIRM_OVERLAY, Confirm};
use crate::screens::text_input::TextRequest;
use crate::ui::fonts::Fonts;
use crate::ui::toast::Toast;

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
    pub nav: NavInput,
    pub log: &'a mut SessionLog,
    pub modes: &'a mut PlayModes,
    pub seen: &'a mut SeenLines,
    pub(crate) screenshot_request: &'a mut bool,
    pub(crate) effects: &'a mut crate::frame::effects::ScreenEffects,
    pub(crate) post: &'a mut crate::frame::post::PostChain,
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

    pub fn shader(&mut self, name: &str, on: bool) {
        if !self.post.set_enabled(name, on) {
            eprintln!("⚠️ No shader named '{}'", name);
        }
    }

    pub fn shader_amount(&mut self, name: &str, amount: f32) {
        if !self.post.set_amount(name, amount) {
            eprintln!("⚠️ No shader named '{}'", name);
        }
    }

    pub fn shake(&mut self, seconds: f32) {
        let now = self.rl.get_time();
        self.effects.shake(now, seconds);
    }

    pub fn flash(&mut self, seconds: f32) {
        let now = self.rl.get_time();
        self.effects.flash(now, seconds);
    }

    pub fn notify(&mut self, text: impl Into<String>) {
        let text = text.into();
        let text = self.label(&text).to_string();
        *self.toast = Some(Toast::info(text));
    }

    pub fn notify_error(&mut self, text: impl Into<String>) {
        let text = text.into();
        let text = self.label(&text).to_string();
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
        crate::data::saves::save_game(
            self.saves,
            slot,
            crate::data::saves::SaveParts {
                story: self.story,
                state: self.state,
                rollback: self.rollback,
                log: self.log.entries(),
                thumbnail: self.thumbnail,
            },
        )
    }

    pub fn screenshot(&mut self) {
        *self.screenshot_request = true;
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

    pub fn set_language(&mut self, code: Option<&str>) {
        self.settings
            .update(|values| values.language = code.map(str::to_string));
        self.apply_language();
    }

    pub fn apply_language(&mut self) {
        let code = self.settings.values.language.clone();
        let catalog = code.and_then(|code| {
            match crate::game::language::load_catalog(self.resources.assets(), &code) {
                Ok(catalog) => Some(catalog),
                Err(e) => {
                    eprintln!("⚠️ {}; playing in the source language", e);
                    None
                }
            }
        });
        if code_missing(&self.settings.values.language, &catalog) {
            self.notify_error("That language could not be loaded");
        }
        self.resources
            .fonts
            .set_language(catalog.as_ref().map(|catalog| catalog.language.as_str()));
        self.story.set_catalog(catalog);
    }

    pub fn label<'a>(&'a self, text: &'a str) -> &'a str {
        crate::ui::labels::label(self.story, text)
    }

    pub fn message(&self, text: &str, fields: &[(&str, &str)]) -> String {
        crate::ui::labels::fill(self.label(text), fields)
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
        let report = crate::data::saves::apply(&file, self.story, self.state)?;
        #[cfg(feature = "character-visuals")]
        self.resources.visuals.reset();
        self.log.replace(file.log.clone());
        self.modes.skip = false;

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

fn code_missing(code: &Option<String>, catalog: &Option<vn_script::Catalog>) -> bool {
    code.is_some() && catalog.is_none()
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
    pub focus_visible: bool,
    pub log: &'a SessionLog,
    pub modes: PlayModes,
}

impl DrawContext<'_> {
    pub fn pointer_over(&self, rl: &RaylibHandle, rect: Rectangle) -> bool {
        self.interactive && crate::ui::is_hovered(rl, rect)
    }

    pub fn shows_focus(&self, focus: &Focus, index: usize) -> bool {
        self.interactive && self.focus_visible && focus.is(index)
    }

    pub fn label<'a>(&'a self, text: &'a str) -> &'a str {
        crate::ui::labels::label(self.story, text)
    }

    pub fn message(&self, text: &str, fields: &[(&str, &str)]) -> String {
        crate::ui::labels::fill(self.label(text), fields)
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
