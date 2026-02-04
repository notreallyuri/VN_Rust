use raylib::prelude::*;

use vn_core::runtime::{ScreenState, providers::ScreenStateManager};

mod components;
mod screen_manager;
mod screens;

pub struct GodIsWatching;

fn main() {
    let (mut rl, thread) = init().size(1280, 720).title("God Is Watching").build();

    let director = Box::new(GodIsWatching);

    let mut state_manager = ScreenStateManager::new(
        ScreenState::StartScreen,
        director,
        "assets/scripts/001.story",
    );

    while !rl.window_should_close() {
        state_manager.update(&mut rl, &thread);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        state_manager.draw(&mut d);
    }
}
