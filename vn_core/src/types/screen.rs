use crate::{
    providers::{ResourceProvider, StoryProvider},
    types::GameContext,
};

#[derive(Clone)]
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

pub trait Screen<U, D, R> {
    fn update(&mut self, ctx: GameContext<U, R>) -> Option<ScreenState>;
    fn draw(&self, target: &mut D, resources: &mut R, story: &StoryProvider);
}
