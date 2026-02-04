use std::collections::HashMap;

use raylib::prelude::*;
use vn_core::{
    runtime::{
        ResourceManager,
        types::{GameContext, Screen, ScreenState},
    },
    script::{Instruction, provider::StoryProvider},
};

pub struct PlayingScreen {
    pub active_characters: HashMap<String, String>,
    pub current_background: String,
    pub current_instruction_index: usize,
}

impl PlayingScreen {
    pub fn new() -> Self {
        Self {
            active_characters: HashMap::new(),
            current_background: String::from(""),
            current_instruction_index: 0,
        }
    }
}

impl Screen for PlayingScreen {
    fn update(&mut self, ctx: GameContext) -> Option<ScreenState> {
        if ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
        {
            ctx.story.next();
        }

        for (name, expression) in &ctx.story.active_characters {
            let path = format!("assets/characters/{}/{}.png", name, expression).to_lowercase();
            ctx.resources.get_or_load(&path, ctx.rl, ctx.thread);
        }

        None
    }

    fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        resources: &mut ResourceManager,
        story: &StoryProvider,
    ) {
        for (name, expression) in &story.active_characters {
            let path = format!("assets/characters/{}/{}.png", name, expression).to_lowercase();

            if let Some(tex) = resources.textures.get(&path) {
                // Hardcoded x for now, or you can add Position to active_characters map
                d.draw_texture(tex, 540, 200, Color::WHITE);
            }
        }

        // 3. Draw Dialogue Box
        // We look at the instruction the VM is CURRENTLY stopped at
        if let Some(Instruction::Say { char_id, text }) = story.instructions.get(story.ip) {
            d.draw_rectangle(50, 550, 1180, 140, Color::new(0, 0, 0, 200));

            if let Some(name) = char_id {
                d.draw_text(name, 70, 560, 20, Color::GOLD);
            }
            d.draw_text(text, 70, 600, 22, Color::RAYWHITE);
        }
    }
}
