use std::rc::Rc;

use raylib::prelude::*;

use crate::context::{DrawContext, GameContext};
use crate::frame::scenery::Scenery;
use crate::screen::{Screen, ScreenState};
use crate::ui;
use crate::ui::fonts::{FontRole, Fonts};
use crate::ui::{Background, TextStyle};

#[derive(Clone, Debug)]
pub struct StartScreenConfig {
    pub title: Option<String>,
    pub title_text: TextStyle,
    pub title_y: f32,
    pub subtitle: Option<String>,
    pub subtitle_text: TextStyle,
    pub scenery: Scenery,
    pub prompt_in_bar: bool,
    pub prompt_pulse: f64,
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
            title: None,
            title_text: TextStyle::new(FontRole::Title, 64.0, Color::RAYWHITE),
            title_y: 0.25,
            subtitle: None,
            subtitle_text: TextStyle::new(FontRole::Menu, 20.0, Color::LIGHTGRAY),
            scenery: Scenery::default(),
            prompt_in_bar: false,
            prompt_pulse: 0.0,
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
    pub fn title(mut self, text: impl Into<String>) -> Self {
        self.title = Some(text.into());
        self
    }

    pub fn title_text(mut self, style: TextStyle) -> Self {
        self.title_text = style;
        self
    }

    pub fn title_y(mut self, fraction: f32) -> Self {
        self.title_y = fraction;
        self
    }

    pub fn subtitle(mut self, text: impl Into<String>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    pub fn subtitle_text(mut self, style: TextStyle) -> Self {
        self.subtitle_text = style;
        self
    }

    pub fn scenery(mut self, scenery: impl FnOnce(Scenery) -> Scenery) -> Self {
        self.scenery = scenery(self.scenery);
        self
    }

    pub fn prompt_in_bar(mut self, in_bar: bool) -> Self {
        self.prompt_in_bar = in_bar;
        self
    }

    pub fn prompt_pulse(mut self, seconds: f64) -> Self {
        self.prompt_pulse = seconds.max(0.0);
        self
    }

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
    opened: Option<f64>,
}

impl StartScreen {
    pub fn new(config: Rc<StartScreenConfig>) -> Self {
        Self {
            config,
            opened: None,
        }
    }
}

pub fn title_block(
    fonts: &Fonts,
    screen: Vector2,
    title: (&str, &TextStyle),
    subtitle: Option<(&str, &TextStyle)>,
    center_y: f32,
) -> Vec<(Vector2, Vector2)> {
    let size = ui::measure_text(fonts, title.0, title.1);
    let mut rects = vec![(
        Vector2::new((screen.x - size.x) / 2.0, center_y - size.y / 2.0),
        size,
    )];
    if let Some((text, style)) = subtitle {
        let sub = ui::measure_text(fonts, text, style);
        rects.push((
            Vector2::new(
                (screen.x - sub.x) / 2.0,
                center_y + size.y / 2.0 + sub.y * 0.4,
            ),
            sub,
        ));
    }
    rects
}

impl Screen for StartScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, self.config.background.as_ref());
        let now = ctx.rl.get_time();
        let opened = *self.opened.get_or_insert(now);
        if now - opened < 0.3 {
            return None;
        }

        let pressed = ctx.rl.get_key_pressed().is_some()
            || ctx.nav.any_key()
            || ctx
                .rl
                .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        pressed.then(|| self.config.next.clone())
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);

        config
            .scenery
            .draw_background(d, ctx.resources, config.background.as_ref());
        let now = d.get_time();
        let since_open = self.opened.map_or(0.0, |at| now - at);
        config.scenery.draw_frame(d, since_open);

        let fonts = ctx.fonts();
        if let Some(title) = &config.title {
            let subtitle = config
                .subtitle
                .as_deref()
                .map(|s| (s, &config.subtitle_text));
            let rects = title_block(
                fonts,
                screen,
                (title, &config.title_text),
                subtitle,
                screen.y * config.title_y,
            );
            let texts = [(title.as_str(), &config.title_text)]
                .into_iter()
                .chain(subtitle);
            for ((position, _), (text, style)) in rects.into_iter().zip(texts) {
                ui::draw_text(d, fonts, ctx.label(text), position, style);
            }
        }

        let center = match config.scenery.bottom_bar(screen, f64::MAX) {
            Some(bar) if config.prompt_in_bar => {
                Vector2::new(screen.x / 2.0, bar.y + bar.height / 2.0)
            }
            _ => Vector2::new(screen.x / 2.0, screen.y * config.prompt_y),
        };
        let mut prompt = config.prompt_text.clone();
        if config.prompt_pulse > 0.0 {
            let wave = (std::f64::consts::TAU * now / config.prompt_pulse).cos() as f32;
            let level = 0.65 + 0.35 * wave;
            prompt.color = prompt.color.alpha(level * prompt.color.a as f32 / 255.0);
        }
        let text = ctx.prompt(&config.prompt);
        ui::draw_text_centered(d, fonts, &text, center, &prompt);

        if let Some(footer) = &config.footer {
            let size = fonts.measure(config.footer_text.font, footer, config.footer_text.size);
            let margin = 12.0;
            let position = Vector2::new(screen.x - size.x - margin, screen.y - size.y - margin);
            ui::draw_text(d, fonts, ctx.label(footer), position, &config.footer_text);
        }
    }
}
