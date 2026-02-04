use raylib::prelude::RaylibDrawHandle;

use crate::{
    runtime::{ResourceManager, types::GameContext},
    script::provider::StoryProvider,
};

#[derive(Clone, Debug)]
pub enum ScreenState {
    StartScreen,
    MainMenu,
    Playing,
    PlayingAlt(String),
    Paused,
    Loading(String),
    Ending,
    Settings(String),
}

pub trait Screen {
    fn update(&mut self, ctx: GameContext) -> Option<ScreenState>;
    fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        resources: &mut ResourceManager,
        story: &StoryProvider,
    );
}
