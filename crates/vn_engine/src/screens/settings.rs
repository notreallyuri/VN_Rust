use std::rc::Rc;

use raylib::prelude::*;

use crate::screens::Typewriter;
use crate::ui::{self, Background, ButtonStyle, TextStyle};
use crate::{
    DrawContext, FontRole, GameContext, Overlay, OverlayAction, Screen, ScreenState, Settings,
};

pub const SETTINGS_OVERLAY: &str = "settings";

const PREVIEW_PAUSE: f64 = 1.5;

#[derive(Clone, Debug)]
pub struct SettingsConfig {
    pub title: String,
    pub title_text: TextStyle,
    pub label_text: TextStyle,
    pub value_button: ButtonStyle,
    pub row_width: f32,
    pub row_spacing: f32,
    pub display_label: String,
    pub windowed_label: String,
    pub fullscreen_label: String,
    pub text_speed_label: String,
    pub text_speeds: Vec<(String, u32)>,
    pub sample_text: String,
    pub sample_text_style: TextStyle,
    pub sample_box_color: Color,
    pub back_button: ButtonStyle,
    pub back_label: String,
    pub back_keys: Vec<KeyboardKey>,
    pub backdrop: Color,
    pub background: Option<Background>,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            title: "Settings".to_string(),
            title_text: TextStyle::new(FontRole::Title, 44.0, Color::RAYWHITE),
            label_text: TextStyle::new(FontRole::Menu, 24.0, Color::RAYWHITE),
            value_button: ButtonStyle::default().size(220.0, 46.0).font_size(20.0),
            row_width: 560.0,
            row_spacing: 16.0,
            display_label: "Display".to_string(),
            windowed_label: "Windowed".to_string(),
            fullscreen_label: "Fullscreen".to_string(),
            text_speed_label: "Text speed".to_string(),
            text_speeds: vec![
                ("Slow".to_string(), 20),
                ("Normal".to_string(), 40),
                ("Fast".to_string(), 80),
                ("Instant".to_string(), 0),
            ],
            sample_text: "This is how fast the story's text appears.".to_string(),
            sample_text_style: TextStyle::new(FontRole::Dialogue, 22.0, Color::RAYWHITE),
            sample_box_color: Color::new(0, 0, 0, 170),
            back_button: ButtonStyle::default().size(200.0, 48.0),
            back_label: "Back".to_string(),
            back_keys: vec![KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_BACKSPACE],
            backdrop: Color::new(0, 0, 0, 200),
            background: None,
        }
    }
}

impl SettingsConfig {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn title_text(mut self, style: TextStyle) -> Self {
        self.title_text = style;
        self
    }

    pub fn label_text(mut self, style: TextStyle) -> Self {
        self.label_text = style;
        self
    }

    pub fn value_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.value_button = style(self.value_button);
        self
    }

    pub fn row_width(mut self, width: f32) -> Self {
        self.row_width = width;
        self
    }

    pub fn row_spacing(mut self, spacing: f32) -> Self {
        self.row_spacing = spacing;
        self
    }

    pub fn text_speeds<S: Into<String>>(
        mut self,
        speeds: impl IntoIterator<Item = (S, u32)>,
    ) -> Self {
        self.text_speeds = speeds
            .into_iter()
            .map(|(label, speed)| (label.into(), speed))
            .collect();
        self
    }

    pub fn sample_text(mut self, text: impl Into<String>) -> Self {
        self.sample_text = text.into();
        self
    }

    pub fn sample_text_style(mut self, style: TextStyle) -> Self {
        self.sample_text_style = style;
        self
    }

    pub fn back_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.back_button = style(self.back_button);
        self
    }

    pub fn back_label(mut self, text: impl Into<String>) -> Self {
        self.back_label = text.into();
        self
    }

    pub fn back_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.back_keys = keys.into_iter().collect();
        self
    }

    pub fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = color;
        self
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    pub fn text_speed_name(&self, speed: u32) -> String {
        self.text_speeds
            .iter()
            .find(|(_, s)| *s == speed)
            .map(|(label, _)| label.clone())
            .unwrap_or_else(|| format!("{} chars/s", speed))
    }

    pub fn next_text_speed(&self, speed: u32) -> u32 {
        let next = self
            .text_speeds
            .iter()
            .position(|(_, s)| *s == speed)
            .map_or(0, |i| i + 1);
        self.text_speeds
            .get(next % self.text_speeds.len().max(1))
            .map_or(speed, |(_, s)| *s)
    }

    fn rows(&self, settings: &Settings) -> [(&str, String); 2] {
        let display = if settings.fullscreen {
            &self.fullscreen_label
        } else {
            &self.windowed_label
        };
        [
            (self.display_label.as_str(), display.clone()),
            (
                self.text_speed_label.as_str(),
                self.text_speed_name(settings.text_speed),
            ),
        ]
    }

    fn value_rects(&self, screen: Vector2) -> Vec<Rectangle> {
        let button = &self.value_button;
        let right = (screen.x + self.row_width) / 2.0;
        (0..2)
            .map(|i| {
                Rectangle::new(
                    right - button.width,
                    150.0 + i as f32 * (button.height + self.row_spacing),
                    button.width,
                    button.height,
                )
            })
            .collect()
    }

    fn sample_rect(&self, screen: Vector2) -> Rectangle {
        let rows = self.value_rects(screen);
        let top = rows.last().map_or(150.0, |r| r.y + r.height) + 40.0;
        Rectangle::new(
            (screen.x - self.row_width) / 2.0,
            top,
            self.row_width,
            self.sample_text_style.size * 1.3 * 3.0 + 32.0,
        )
    }

    fn back_rect(&self, screen: Vector2) -> Rectangle {
        let style = &self.back_button;
        Rectangle::new(
            (screen.x - style.width) / 2.0,
            screen.y - style.height - 40.0,
            style.width,
            style.height,
        )
    }
}

enum Outcome {
    Stay,
    Back,
}

struct SettingsMenu {
    config: Rc<SettingsConfig>,
    preview: Option<(Typewriter, u32)>,
    visible: usize,
}

impl SettingsMenu {
    fn new(config: Rc<SettingsConfig>) -> Self {
        Self {
            config,
            preview: None,
            visible: 0,
        }
    }

    fn update(&mut self, ctx: &mut GameContext) -> Outcome {
        let config = Rc::clone(&self.config);
        let screen = ui::screen_size(ctx.rl);

        let back_key = config.back_keys.iter().any(|&k| ctx.rl.is_key_pressed(k));
        if back_key || ui::is_clicked(ctx.rl, config.back_rect(screen)) {
            return Outcome::Back;
        }

        let rects = config.value_rects(screen);
        if ui::is_clicked(ctx.rl, rects[0]) {
            ctx.settings.update(|s| s.fullscreen = !s.fullscreen);
        }
        if ui::is_clicked(ctx.rl, rects[1]) {
            ctx.settings
                .update(|s| s.text_speed = config.next_text_speed(s.text_speed));
        }

        self.animate_preview(ctx.rl.get_time(), ctx.settings.values.text_speed);
        Outcome::Stay
    }

    fn animate_preview(&mut self, now: f64, speed: u32) {
        let text = &self.config.sample_text;
        let restart = match &self.preview {
            Some((_, shown_speed)) if *shown_speed != speed => true,
            Some((typewriter, _)) => typewriter.is_done(now - PREVIEW_PAUSE) && speed != 0,
            None => true,
        };

        if restart {
            self.preview = Some((Typewriter::start(text, speed, now), speed));
        }
        self.visible = self
            .preview
            .as_ref()
            .map_or(0, |(typewriter, _)| typewriter.visible(now));
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();

        ui::draw_text_centered(
            d,
            fonts,
            &config.title,
            Vector2::new(screen.x / 2.0, 70.0),
            &config.title_text,
        );

        let left = (screen.x - config.row_width) / 2.0;
        for ((label, value), rect) in config
            .rows(ctx.settings)
            .iter()
            .zip(config.value_rects(screen))
        {
            let label_y = rect.y + (rect.height - config.label_text.size) / 2.0;
            ui::draw_text(
                d,
                fonts,
                label,
                Vector2::new(left, label_y),
                &config.label_text,
            );
            ui::draw_button(d, fonts, rect, value, &config.value_button);
        }

        let sample = config.sample_rect(screen);
        d.draw_rectangle_rec(sample, config.sample_box_color);
        ui::draw_text_wrapped_visible(
            d,
            fonts,
            &config.sample_text,
            Vector2::new(sample.x + 16.0, sample.y + 16.0),
            sample.width - 32.0,
            &config.sample_text_style,
            self.visible,
        );

        ui::draw_button(
            d,
            fonts,
            config.back_rect(screen),
            &config.back_label,
            &config.back_button,
        );
    }
}

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
