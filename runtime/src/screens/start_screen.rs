use raylib::prelude::*;
use vn_core::types::{Screen, ScreenState};

pub struct StartScreen;

impl StartScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for StartScreen {
    fn update(
        &mut self,
        ctx: &mut vn_core::types::GameContext,
    ) -> Option<vn_core::types::ScreenState> {
        if ctx.rl.get_key_pressed().is_some()
            || ctx
                .rl
                .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
        {
            return Some(ScreenState::MainMenu);
        }
        None
    }

    fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        _thread: &RaylibThread,
        _resources: &mut vn_core::providers::ResourceProvider,
        _story: &vn_core::providers::StoryProvider,
    ) {
        d.draw_text("PRESS ANY KEY TO START", 450, 350, 30, Color::RAYWHITE);

        d.draw_text("v0.1.0", 1210, 690, 10, Color::GRAY);
    }
}
