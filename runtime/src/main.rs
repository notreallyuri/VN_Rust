use raylib::prelude::*;
use vn_core::{
    providers::{ResourceProvider, ScreenProvider, ScreenStateManager},
    types::ScreenState,
};

use crate::screen_manager::RaylibUpdateCtx;

mod components;
mod providers;
mod screen_manager;
mod screens;

pub struct GodIsWatching;

fn main() {
    let (mut rl, thread) = init().size(1280, 720).title("God Is Watching").build();

    let director = GodIsWatching;

    let mut state_manager =
        ScreenStateManager::new(ScreenState::StartScreen, director, "assets/scripts/001.bin");

    while !rl.window_should_close() {
        let mut context = RaylibUpdateCtx {
            rl: &mut rl,
            thread: &thread,
        };

        state_manager.update(&mut context);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        state_manager.draw(&mut d);
    }
}
