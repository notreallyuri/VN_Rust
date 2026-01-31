use raylib::prelude::*;
use vn_core::{
    components::Button,
    providers::{ResourceProvider, StoryProvider},
    types::{GameContext, Screen, ScreenState},
};

pub struct MainMenuScreen {
    buttons: Vec<Button>,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        let mut buttons = Vec::new();

        let center_x = 1280.0 / 2.0 - 100.0;
        let primary_color = Color::new(40, 40, 60, 255);

        buttons.push(Button::new(
            center_x,
            300.0,
            200.0,
            50.0,
            "New Game",
            primary_color,
        ));
        buttons.push(Button::new(
            center_x,
            370.0,
            200.0,
            50.0,
            "Settings",
            primary_color,
        ));
        buttons.push(Button::new(
            center_x,
            440.0,
            200.0,
            50.0,
            "Exit",
            Color::new(80, 30, 30, 255),
        ));

        Self { buttons }
    }
}

impl Screen for MainMenuScreen {
    fn update(&mut self, ctx: &mut GameContext) -> Option<ScreenState> {
        if self.buttons[0].is_clicked(ctx.rl) {
            return Some(ScreenState::Playing);
        }
        if self.buttons[1].is_clicked(ctx.rl) {
            return Some(ScreenState::Settings("display".to_string()));
        }
        if self.buttons[2].is_clicked(ctx.rl) {
            std::process::exit(0);
        }

        None
    }

    fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        _thread: &RaylibThread,
        _resources: &mut ResourceProvider,
        _story: &StoryProvider,
    ) {
        d.draw_text("God Is Watching", 480, 150, 60, Color::RAYWHITE);

        for b in &self.buttons {
            b.draw(d);
        }
    }
}
