use std::rc::Rc;

use raylib::prelude::*;

use crate::ui::{self, Background, TextStyle};
use crate::{DrawContext, FontRole, GameContext, Screen, ScreenState};

#[derive(Clone, Debug)]
pub struct StartScreenConfig {
    pub prompt: String,
    pub prompt_text: TextStyle,
    pub prompt_y: f32,
    pub footer: Option<String>,
    pub footer_text: TextStyle,
    pub background: Option<Background>,
    pub next: ScreenState,
}

impl Default for StartScreenConfig {
    fn default() -> Self {
        Self {
            prompt: "Press any key to start".to_string(),
            prompt_text: TextStyle::new(FontRole::Menu, 30.0, Color::RAYWHITE),
            prompt_y: 0.5,
            footer: None,
            footer_text: TextStyle::new(FontRole::Menu, 14.0, Color::GRAY),
            background: None,
            next: ScreenState::MainMenu,
        }
    }
}

impl StartScreenConfig {
    pub fn prompt(mut self, text: impl Into<String>) -> Self {
        self.prompt = text.into();
        self
    }

    pub fn prompt_text(mut self, style: TextStyle) -> Self {
        self.prompt_text = style;
        self
    }

    pub fn prompt_y(mut self, fraction: f32) -> Self {
        self.prompt_y = fraction;
        self
    }

    pub fn footer(mut self, text: impl Into<String>) -> Self {
        self.footer = Some(text.into());
        self
    }

    pub fn footer_text(mut self, style: TextStyle) -> Self {
        self.footer_text = style;
        self
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    pub fn next(mut self, state: ScreenState) -> Self {
        self.next = state;
        self
    }
}

pub struct StartScreen {
    config: Rc<StartScreenConfig>,
}

impl StartScreen {
    pub fn new(config: Rc<StartScreenConfig>) -> Self {
        Self { config }
    }
}

impl Screen for StartScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, self.config.background.as_ref());

        let pressed = ctx.rl.get_key_pressed().is_some()
            || ctx
                .rl
                .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        pressed.then(|| self.config.next.clone())
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);

        ui::draw_background(d, ctx.resources, config.background.as_ref());

        let fonts = ctx.fonts();
        ui::draw_text_centered(
            d,
            fonts,
            &config.prompt,
            Vector2::new(screen.x / 2.0, screen.y * config.prompt_y),
            &config.prompt_text,
        );

        if let Some(footer) = &config.footer {
            let size = fonts.measure(config.footer_text.font, footer, config.footer_text.size);
            let margin = 12.0;
            let position = Vector2::new(screen.x - size.x - margin, screen.y - size.y - margin);
            ui::draw_text(d, fonts, footer, position, &config.footer_text);
        }
    }
}
