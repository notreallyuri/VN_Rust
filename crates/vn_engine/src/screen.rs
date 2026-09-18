use raylib::prelude::RaylibDrawHandle;

use crate::{DrawContext, GameContext};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ScreenState {
    StartScreen,
    MainMenu,
    Playing,
    Save,
    Load,
    TextInput,
    Settings,
    Custom(String),
    Quit,
}

pub trait Screen {
    fn update(&mut self, ctx: GameContext) -> Option<ScreenState>;
    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext);
}
