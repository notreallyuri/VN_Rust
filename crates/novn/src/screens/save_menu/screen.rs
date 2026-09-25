use std::rc::Rc;

use raylib::prelude::*;

use super::menu::{Outcome, SaveMenu};
use super::{SaveMenuConfig, SaveMenuMode};
use crate::context::{DrawContext, GameContext};
use crate::overlay::{Overlay, OverlayAction};
use crate::screen::{Screen, ScreenState};
use crate::ui;

pub struct SaveMenuScreen {
    menu: SaveMenu,
}

impl SaveMenuScreen {
    pub fn new(config: Rc<SaveMenuConfig>, mode: SaveMenuMode) -> Self {
        Self {
            menu: SaveMenu::new(config, mode, false),
        }
    }
}

impl Screen for SaveMenuScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, self.menu.config.background.as_ref());

        match self.menu.update(&mut ctx) {
            Outcome::Stay => None,
            Outcome::Loaded => Some(ScreenState::Playing),
            Outcome::Back => Some(match ctx.previous {
                Some(state) if !matches!(state, ScreenState::Save | ScreenState::Load) => {
                    state.clone()
                }
                _ => ScreenState::MainMenu,
            }),
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        ui::draw_background(d, ctx.resources, self.menu.config.background.as_ref());
        self.menu.draw(d, ctx);
    }
}

pub struct SaveMenuOverlay {
    menu: SaveMenu,
}

impl SaveMenuOverlay {
    pub fn new(config: Rc<SaveMenuConfig>, mode: SaveMenuMode) -> Self {
        Self {
            menu: SaveMenu::new(config, mode, true),
        }
    }
}

impl Overlay for SaveMenuOverlay {
    fn update(&mut self, mut ctx: GameContext) -> OverlayAction {
        match self.menu.update(&mut ctx) {
            Outcome::Stay => OverlayAction::Stay,
            Outcome::Back => OverlayAction::Close,
            Outcome::Loaded => OverlayAction::Goto(ScreenState::Playing),
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        d.draw_rectangle(
            0,
            0,
            screen.x as i32,
            screen.y as i32,
            self.menu.config.backdrop,
        );
        self.menu.draw(d, ctx);
    }
}
