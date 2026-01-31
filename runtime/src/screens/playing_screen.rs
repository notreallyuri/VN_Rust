use raylib::prelude::*;
use vn_core::{
    providers::{ResourceProvider, StoryProvider},
    types::{GameContext, Screen, ScreenState},
};

pub struct PlayingScreen;

impl PlayingScreen {
    pub fn new() -> Self {
        Self
    }
}

impl Screen for PlayingScreen {
    fn update(&mut self, ctx: &mut GameContext) -> Option<ScreenState> {
        if ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
        {
            ctx.story.next_line();
        }
        None
    }

    fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        thread: &RaylibThread,
        resources: &mut ResourceProvider,
        story: &StoryProvider,
    ) {
        for (path, pos) in story.get_render_data() {
            let tex = resources.get_texture(d, thread, &path);
            let x = match pos {
                vn_core::types::Position::Left => 100,
                vn_core::types::Position::Middle => 540,
                vn_core::types::Position::Right => 900,
            };
            d.draw_texture(tex, x, 200, Color::WHITE);
        }

        let line = &story.current_scene.lines[story.current_index];
        d.draw_rectangle(50, 550, 1180, 140, Color::new(0, 0, 0, 200));

        if let Some(name) = &line.character {
            d.draw_text(name, 70, 560, 20, Color::GOLD);
        }
        d.draw_text(&line.text, 70, 600, 22, Color::RAYWHITE);
    }
}
