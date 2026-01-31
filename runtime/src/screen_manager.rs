use vn_core::providers::ScreenProvider;
use vn_core::types::ScreenState;

use crate::GodIsWatching;
use crate::screens::main_menu::MainMenuScreen;
use crate::screens::playing_screen::PlayingScreen;
use crate::screens::start_screen::StartScreen;

impl ScreenProvider for GodIsWatching {
    fn create_screen(&self, state: &ScreenState) -> Box<dyn vn_core::types::Screen> {
        match state {
            ScreenState::StartScreen => Box::new(StartScreen::new()),
            ScreenState::MainMenu => Box::new(MainMenuScreen::new()),
            ScreenState::Playing => Box::new(PlayingScreen::new()),
            _ => Box::new(StartScreen::new()),
        }
    }
}
