use vn_core::runtime::{
    ScreenFactory,
    types::{Screen, ScreenState},
};

use crate::{
    GodIsWatching,
    screens::{
        main_menu::MainMenuScreen, playing_screen::PlayingScreen, start_screen::StartScreen,
    },
};

impl ScreenFactory for GodIsWatching {
    fn create_screen(&self, state: &ScreenState) -> Box<dyn Screen> {
        match state {
            ScreenState::StartScreen => Box::new(StartScreen::new()),
            ScreenState::MainMenu => Box::new(MainMenuScreen::new()),
            ScreenState::Playing => Box::new(PlayingScreen::new()),
            _ => Box::new(StartScreen::new()),
        }
    }
}
