use raylib::prelude::RaylibDrawHandle;

use crate::context::{DrawContext, GameContext};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ScreenState {
    StartScreen,
    MainMenu,
    Playing,
    Save,
    Load,
    TextInput,
    #[cfg(any(feature = "video-portable", feature = "video-ffmpeg"))]
    Video,
    Settings,
    Custom(String),
    Quit,
}

pub trait Screen {
    fn set_paused(&mut self, _paused: bool, _now: f64) {}

    fn update(&mut self, ctx: GameContext) -> Option<ScreenState>;
    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext);
}
