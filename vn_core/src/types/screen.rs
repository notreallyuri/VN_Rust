use raylib::prelude::*;

use crate::providers::{ResourceProvider, StoryProvider};

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

pub trait Screen {
    fn update(&mut self, _ctx: &mut GameContext) -> Option<ScreenState> {
        None
    }

    fn draw(
        &self,
        _d: &mut RaylibDrawHandle,
        _thread: &RaylibThread,
        _resources: &mut ResourceProvider,
        _story: &StoryProvider,
    ) {
    }
}

pub struct GameContext<'a> {
    pub rl: &'a mut RaylibHandle,
    pub thread: &'a RaylibThread,
    pub resources: &'a mut ResourceProvider,
    pub story: &'a mut StoryProvider,
}
