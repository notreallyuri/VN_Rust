use std::rc::Rc;

use raylib::prelude::*;
use vn_script::Value;

use crate::ui::{self, Background, TextStyle};
use crate::{DrawContext, FontRole, GameContext, Screen, ScreenState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextRequest {
    pub variable: String,
    pub prompt: String,
    pub initial: String,
    pub max_len: usize,
    pub allow_empty: bool,
}

impl TextRequest {
    pub fn new(variable: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            variable: variable.into(),
            prompt: prompt.into(),
            initial: String::new(),
            max_len: 24,
            allow_empty: false,
        }
    }

    pub fn initial(mut self, text: impl Into<String>) -> Self {
        self.initial = text.into();
        self
    }

    pub fn max_len(mut self, max_len: usize) -> Self {
        self.max_len = max_len.max(1);
        self
    }

    pub fn allow_empty(mut self, allow: bool) -> Self {
        self.allow_empty = allow;
        self
    }
}

#[derive(Clone, Debug)]
pub struct TextInputConfig {
    pub prompt_text: TextStyle,
    pub input_text: TextStyle,
    pub hint: String,
    pub hint_text: TextStyle,
    pub box_width: f32,
    pub box_height: f32,
    pub box_color: Color,
    pub box_border: Color,
    pub background: Option<Background>,
}

impl Default for TextInputConfig {
    fn default() -> Self {
        Self {
            prompt_text: TextStyle::new(FontRole::Title, 36.0, Color::RAYWHITE),
            input_text: TextStyle::new(FontRole::Dialogue, 30.0, Color::RAYWHITE),
            hint: "Type, then press Enter".to_string(),
            hint_text: TextStyle::new(FontRole::Menu, 16.0, Color::GRAY),
            box_width: 520.0,
            box_height: 60.0,
            box_color: Color::new(20, 20, 30, 230),
            box_border: Color::new(255, 255, 255, 60),
            background: None,
        }
    }
}

impl TextInputConfig {
    pub fn prompt_text(mut self, style: TextStyle) -> Self {
        self.prompt_text = style;
        self
    }

    pub fn input_text(mut self, style: TextStyle) -> Self {
        self.input_text = style;
        self
    }

    pub fn hint(mut self, text: impl Into<String>) -> Self {
        self.hint = text.into();
        self
    }

    pub fn hint_text(mut self, style: TextStyle) -> Self {
        self.hint_text = style;
        self
    }

    pub fn box_size(mut self, width: f32, height: f32) -> Self {
        self.box_width = width;
        self.box_height = height;
        self
    }

    pub fn box_color(mut self, color: Color) -> Self {
        self.box_color = color;
        self
    }

    pub fn box_border(mut self, color: Color) -> Self {
        self.box_border = color;
        self
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }
}

pub struct TextInputScreen {
    config: Rc<TextInputConfig>,
    request: Option<TextRequest>,
    text: String,
    error: Option<String>,
    time: f64,
}

impl TextInputScreen {
    pub fn new(config: Rc<TextInputConfig>) -> Self {
        Self {
            config,
            request: None,
            text: String::new(),
            error: None,
            time: 0.0,
        }
    }

    fn submit(&mut self, ctx: &mut GameContext) -> Option<ScreenState> {
        let request = self.request.as_ref()?;
        let value = self.text.trim().to_string();

        if value.is_empty() && !request.allow_empty {
            self.error = Some("Please type something.".to_string());
            return None;
        }

        match ctx
            .story
            .set_variable(request.variable.clone(), Value::String(value))
        {
            Ok(()) => Some(ScreenState::Playing),
            Err(e) => {
                eprintln!("⚠️ Text input: {}", e);
                Some(ScreenState::Playing)
            }
        }
    }
}

impl Screen for TextInputScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, self.config.background.as_ref());
        self.time = ctx.rl.get_time();

        if self.request.is_none() {
            match ctx.take_text_request() {
                Some(request) => {
                    self.text = request.initial.clone();
                    self.request = Some(request);
                }
                None => return Some(ScreenState::Playing),
            }
        }
        let max_len = self.request.as_ref().map_or(0, |r| r.max_len);

        while let Some(c) = ctx.rl.get_char_pressed() {
            if !c.is_control() && self.text.chars().count() < max_len {
                self.text.push(c);
                self.error = None;
            }
        }

        if ctx.rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
            || ctx.rl.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE)
        {
            self.text.pop();
        }

        if ctx.rl.is_key_pressed(KeyboardKey::KEY_ENTER)
            || ctx.rl.is_key_pressed(KeyboardKey::KEY_KP_ENTER)
        {
            return self.submit(&mut ctx);
        }

        None
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();

        ui::draw_background(d, ctx.resources, config.background.as_ref());

        let prompt = self.request.as_ref().map_or("", |r| r.prompt.as_str());
        ui::draw_text_centered(
            d,
            fonts,
            prompt,
            Vector2::new(screen.x / 2.0, screen.y * 0.38),
            &config.prompt_text,
        );

        let field = Rectangle::new(
            (screen.x - config.box_width) / 2.0,
            screen.y * 0.47,
            config.box_width,
            config.box_height,
        );
        d.draw_rectangle_rec(field, config.box_color);
        d.draw_rectangle_lines_ex(field, 2.0, config.box_border);

        let style = &config.input_text;
        let width = fonts.measure(style.font, &self.text, style.size).x;
        let text_x = field.x + (field.width - width) / 2.0;
        let text_y = field.y + (field.height - style.size) / 2.0;
        ui::draw_text(d, fonts, &self.text, Vector2::new(text_x, text_y), style);

        if (self.time * 2.0) as i64 % 2 == 0 {
            let cursor_x = text_x + width + 3.0;
            d.draw_rectangle(
                cursor_x as i32,
                text_y as i32,
                2,
                style.size as i32,
                style.color,
            );
        }

        let (hint, hint_style) = match &self.error {
            Some(error) => (
                error.as_str(),
                config
                    .hint_text
                    .clone()
                    .color(Color::new(230, 110, 110, 255)),
            ),
            None => (config.hint.as_str(), config.hint_text.clone()),
        };
        ui::draw_text_centered(
            d,
            fonts,
            hint,
            Vector2::new(screen.x / 2.0, field.y + field.height + 30.0),
            &hint_style,
        );
    }
}
