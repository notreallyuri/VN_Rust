use vn_core::{
    providers::ScreenProvider,
    types::{Screen, ScreenState},
};

use crate::{
    GodIsWatching,
    providers::{raylib_context::RaylibCtx, resource::RaylibTextureManager},
    screens::{
        main_menu::MainMenuScreen, playing_screen::PlayingScreen, start_screen::StartScreen,
    },
};

use raylib::prelude::*;

pub struct RaylibDrawCtx<'a> {
    pub d: &'a mut RaylibDrawHandle<'a>,
    pub thread: &'a RaylibThread,
}

impl ScreenProvider<RaylibCtx, RaylibCtx, RaylibTextureManager> for GodIsWatching {
    fn create_screen(
        &self,
        state: &ScreenState,
    ) -> Box<dyn Screen<RaylibCtx, RaylibCtx, RaylibTextureManager>> {
        match state {
            ScreenState::StartScreen => Box::new(StartScreen::new()),
            ScreenState::MainMenu => Box::new(MainMenuScreen::new()),
            ScreenState::Playing => Box::new(PlayingScreen),
            _ => Box::new(StartScreen::new()),
        }
    }
}
