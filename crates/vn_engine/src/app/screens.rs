use std::collections::HashMap;
use std::rc::Rc;

use super::{OverlayBuilder, ScreenBuilder};
use crate::screens::{
    CONFIRM_OVERLAY, ConfirmConfig, ConfirmDialog, KEYBINDS_OVERLAY, KeybindsConfig,
    KeybindsOverlay, LOAD_OVERLAY, LOG_OVERLAY, LogConfig, LogOverlay, MainMenuConfig,
    MainMenuScreen, PAUSE_OVERLAY, PauseMenu, PauseMenuConfig, PlayingConfig, PlayingScreen,
    SAVE_OVERLAY, SETTINGS_OVERLAY, SaveMenuConfig, SaveMenuMode, SaveMenuOverlay, SaveMenuScreen,
    SettingsConfig, SettingsOverlay, SettingsScreen, StartScreen, StartScreenConfig,
    TextInputConfig, TextInputScreen, default_keybinds,
};
use crate::{NavigationConfig, Overlay, RollbackConfig, Screen, ScreenFactory, ScreenState};

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
