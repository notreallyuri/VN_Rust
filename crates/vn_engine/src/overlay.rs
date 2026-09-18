use raylib::prelude::RaylibDrawHandle;

use crate::{DrawContext, GameContext, ScreenState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OverlayAction {
    Stay,
    Close,
    CloseAll,
    Goto(ScreenState),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OverlayRequest {
    Open(String),
    Close,
    CloseAll,
}

pub trait Overlay {
    fn update(&mut self, ctx: GameContext) -> OverlayAction;
    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext);
}
