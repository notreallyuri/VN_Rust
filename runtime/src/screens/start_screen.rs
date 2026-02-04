use raylib::prelude::*;
use vn_core::{
    runtime::{
        ResourceManager,
        types::{GameContext, Screen, ScreenState},
    },
    script::provider::StoryProvider,
};

pub struct StartScreen;

impl StartScreen {
    pub fn new() -> Self {
        Self {}
    }
}

impl Screen for StartScreen {
    fn update(&mut self, ctx: GameContext) -> Option<ScreenState> {
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
        _resources: &mut ResourceManager,
        _story: &StoryProvider,
    ) {
        d.draw_text("PRESS ANY KEY TO START", 450, 350, 30, Color::RAYWHITE);

        d.draw_text("v0.1.0", 1210, 690, 10, Color::GRAY);
    }
}
