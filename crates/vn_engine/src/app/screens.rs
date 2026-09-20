use std::collections::HashMap;
use std::rc::Rc;

use super::{OverlayBuilder, ScreenBuilder};
use crate::data::rollback::RollbackConfig;
use crate::input::navigation::NavigationConfig;
use crate::overlay::Overlay;
use crate::screen::{Screen, ScreenState};
use crate::screen_manager::ScreenFactory;
use crate::screens::confirm::{CONFIRM_OVERLAY, ConfirmConfig, ConfirmDialog};
use crate::screens::keybinds::{
    KEYBINDS_OVERLAY, KeybindsConfig, KeybindsOverlay, default_keybinds,
};
use crate::screens::log::{LOG_OVERLAY, LogConfig, LogOverlay};
use crate::screens::main_menu::{MainMenuConfig, MainMenuScreen};
use crate::screens::pause_menu::{
    LOAD_OVERLAY, PAUSE_OVERLAY, PauseMenu, PauseMenuConfig, SAVE_OVERLAY,
};
use crate::screens::playing::{PlayingConfig, PlayingScreen};
use crate::screens::save_menu::{SaveMenuConfig, SaveMenuMode, SaveMenuOverlay, SaveMenuScreen};
use crate::screens::settings::{SETTINGS_OVERLAY, SettingsConfig, SettingsOverlay, SettingsScreen};
use crate::screens::start::{StartScreen, StartScreenConfig};
use crate::screens::text_input::{TextInputConfig, TextInputScreen};

pub struct DefaultScreens {
    pub title: String,
    pub start: Rc<StartScreenConfig>,
    pub menu: Rc<MainMenuConfig>,
    pub playing: Rc<PlayingConfig>,
    pub save_menu: Rc<SaveMenuConfig>,
    pub text_input: Rc<TextInputConfig>,
    pub pause_menu: Rc<PauseMenuConfig>,
    pub confirm_dialog: Rc<ConfirmConfig>,
    pub settings: Rc<SettingsConfig>,
    pub log: Rc<LogConfig>,
    pub keybinds: Rc<KeybindsConfig>,
    pub rollback: RollbackConfig,
    pub navigation: NavigationConfig,
    pub overrides: HashMap<ScreenState, ScreenBuilder>,
    pub overlays: HashMap<String, OverlayBuilder>,
}

impl ScreenFactory for DefaultScreens {
    fn create_screen(&self, state: &ScreenState) -> Option<Box<dyn Screen>> {
        if let Some(build) = self.overrides.get(state) {
            return Some(build());
        }

        match state {
            ScreenState::StartScreen => Some(Box::new(StartScreen::new(self.start.clone()))),
            ScreenState::MainMenu => Some(Box::new(MainMenuScreen::new(
                self.menu.clone(),
                &self.title,
            ))),
            ScreenState::Playing => Some(Box::new(PlayingScreen::new(self.playing.clone()))),
            ScreenState::Save => Some(Box::new(SaveMenuScreen::new(
                self.save_menu.clone(),
                SaveMenuMode::Save,
            ))),
            ScreenState::Load => Some(Box::new(SaveMenuScreen::new(
                self.save_menu.clone(),
                SaveMenuMode::Load,
            ))),
            ScreenState::TextInput => Some(Box::new(TextInputScreen::new(self.text_input.clone()))),
            ScreenState::Settings => Some(Box::new(SettingsScreen::new(self.settings.clone()))),
            _ => None,
        }
    }

    fn create_overlay(&self, name: &str) -> Option<Box<dyn Overlay>> {
        if let Some(build) = self.overlays.get(name) {
            return Some(build());
        }

        match name {
            PAUSE_OVERLAY => Some(Box::new(PauseMenu::new(self.pause_menu.clone()))),
            CONFIRM_OVERLAY => Some(Box::new(ConfirmDialog::new(self.confirm_dialog.clone()))),
            SETTINGS_OVERLAY => Some(Box::new(SettingsOverlay::new(self.settings.clone()))),
            LOG_OVERLAY => Some(Box::new(LogOverlay::new(self.log.clone()))),
            KEYBINDS_OVERLAY => Some(Box::new(KeybindsOverlay::new(
                self.keybinds.clone(),
                default_keybinds(&self.playing, &self.rollback, &self.navigation),
            ))),
            SAVE_OVERLAY => Some(Box::new(SaveMenuOverlay::new(
                self.save_menu.clone(),
                SaveMenuMode::Save,
            ))),
            LOAD_OVERLAY => Some(Box::new(SaveMenuOverlay::new(
                self.save_menu.clone(),
                SaveMenuMode::Load,
            ))),
            _ => None,
        }
    }
}
