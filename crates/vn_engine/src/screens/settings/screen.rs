use std::rc::Rc;

use raylib::prelude::*;

use super::{Outcome, SettingsConfig, SettingsMenu};
use crate::context::{DrawContext, GameContext};
use crate::overlay::{Overlay, OverlayAction};
use crate::screen::{Screen, ScreenState};
use crate::ui;

pub struct SettingsScreen {
    menu: SettingsMenu,
}

impl SettingsScreen {
    pub fn new(config: Rc<SettingsConfig>) -> Self {
        Self {
            menu: SettingsMenu::new(config),
        }
    }
}

impl Screen for SettingsScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, self.menu.config.background.as_ref());

        match self.menu.update(&mut ctx) {
            Outcome::Stay => None,
            Outcome::Back => Some(match ctx.previous {
                Some(state) if *state != ScreenState::Settings => state.clone(),
                _ => ScreenState::MainMenu,
            }),
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        ui::draw_background(d, ctx.resources, self.menu.config.background.as_ref());
        self.menu.draw(d, ctx);
    }
}

pub struct SettingsOverlay {
    menu: SettingsMenu,
}

impl SettingsOverlay {
    pub fn new(config: Rc<SettingsConfig>) -> Self {
        Self {
            menu: SettingsMenu::new(config),
        }
    }
}

impl Overlay for SettingsOverlay {
    fn update(&mut self, mut ctx: GameContext) -> OverlayAction {
        match self.menu.update(&mut ctx) {
            Outcome::Stay => OverlayAction::Stay,
            Outcome::Back => OverlayAction::Close,
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
