use raylib::prelude::*;

use crate::{FontRole, Fonts, GameContext, ResourceManager};

#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub font: FontRole,
    pub size: f32,
    pub color: Color,
}

impl TextStyle {
    pub fn new(font: FontRole, size: f32, color: Color) -> Self {
        Self { font, size, color }
    }

    pub fn font(mut self, font: FontRole) -> Self {
        self.font = font;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonStyle {
    pub width: f32,
    pub height: f32,
    pub color: Color,
    pub hover_color: Color,
    pub roundness: f32,
    pub text: TextStyle,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        let color = Color::new(40, 40, 60, 255);
        Self {
            width: 240.0,
            height: 52.0,
            color,
            hover_color: lighten(color),
            roundness: 0.0,
            text: TextStyle::new(FontRole::Button, 22.0, Color::WHITE),
        }
    }
}

impl ButtonStyle {
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self.hover_color = lighten(color);
        self
    }

    pub fn hover_color(mut self, color: Color) -> Self {
        self.hover_color = color;
        self
    }

    pub fn roundness(mut self, roundness: f32) -> Self {
        self.roundness = roundness.clamp(0.0, 1.0);
        self
    }

    pub fn text(mut self, text: TextStyle) -> Self {
        self.text = text;
        self
    }

    pub fn font(mut self, font: FontRole) -> Self {
        self.text.font = font;
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.text.size = size;
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text.color = color;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Background {
    Color(Color),
    Image(String),
}

pub fn lighten(color: Color) -> Color {
    let up = |c: u8| c.saturating_add(24);
    Color::new(up(color.r), up(color.g), up(color.b), color.a)
}

pub fn is_hovered(rl: &RaylibHandle, rect: Rectangle) -> bool {
    rect.check_collision_point_rec(rl.get_mouse_position())
}

pub fn is_clicked(rl: &RaylibHandle, rect: Rectangle) -> bool {
    is_hovered(rl, rect) && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
}

pub fn draw_button(
    d: &mut RaylibDrawHandle,
    fonts: &Fonts,
    rect: Rectangle,
    label: &str,
    style: &ButtonStyle,
) {
    let color = if is_hovered(d, rect) {
        style.hover_color
    } else {
        style.color
    };

    if style.roundness > 0.0 {
        d.draw_rectangle_rounded(rect, style.roundness, 8, color);
    } else {
        d.draw_rectangle_rec(rect, color);
    }

    let text_size = fonts.measure(style.text.font, label, style.text.size);
    let position = Vector2::new(
        rect.x + (rect.width - text_size.x) / 2.0,
        rect.y + (rect.height - text_size.y) / 2.0,
    );
    draw_text(d, fonts, label, position, &style.text);
}

pub fn draw_text(
    d: &mut RaylibDrawHandle,
    fonts: &Fonts,
    text: &str,
    position: Vector2,
    style: &TextStyle,
) {
    fonts.draw(d, style.font, text, position, style.size, style.color);
}

pub fn draw_text_centered(
    d: &mut RaylibDrawHandle,
    fonts: &Fonts,
    text: &str,
    center: Vector2,
    style: &TextStyle,
) {
    let size = fonts.measure(style.font, text, style.size);
    let position = Vector2::new(center.x - size.x / 2.0, center.y - size.y / 2.0);
    draw_text(d, fonts, text, position, style);
}

pub fn draw_text_wrapped(
    d: &mut RaylibDrawHandle,
    fonts: &Fonts,
    text: &str,
    position: Vector2,
    max_width: f32,
    style: &TextStyle,
) -> f32 {
    let line_height = style.size * 1.3;
    let lines = fonts.wrap(style.font, text, style.size, max_width);

    for (i, line) in lines.iter().enumerate() {
        let y = position.y + i as f32 * line_height;
        draw_text(d, fonts, line, Vector2::new(position.x, y), style);
    }

    lines.len() as f32 * line_height
}

pub fn draw_text_wrapped_visible(
    d: &mut RaylibDrawHandle,
    fonts: &Fonts,
    text: &str,
    position: Vector2,
    max_width: f32,
    style: &TextStyle,
    visible: usize,
) -> f32 {
    let line_height = style.size * 1.3;
    let lines = fonts.wrap(style.font, text, style.size, max_width);
    let mut remaining = visible;

    for (i, line) in lines.iter().enumerate() {
        if remaining == 0 {
            break;
        }
        let shown: String = line.chars().take(remaining).collect();
        remaining = remaining.saturating_sub(line.chars().count() + 1);
        let y = position.y + i as f32 * line_height;
        draw_text(d, fonts, &shown, Vector2::new(position.x, y), style);
    }

    lines.len() as f32 * line_height
}

pub fn screen_size(rl: &RaylibHandle) -> Vector2 {
    Vector2::new(rl.get_screen_width() as f32, rl.get_screen_height() as f32)
}

pub fn load_background(ctx: &mut GameContext, background: Option<&Background>) {
    if let Some(Background::Image(path)) = background {
        ctx.resources.get_or_load(path, ctx.rl, ctx.thread);
    }
}

pub fn draw_background(
    d: &mut RaylibDrawHandle,
    resources: &ResourceManager,
    background: Option<&Background>,
) {
    let screen = screen_size(d);

    match background {
        Some(Background::Color(color)) => d.clear_background(*color),
        Some(Background::Image(path)) => {
            if let Some(texture) = resources.textures.get(path) {
                let source = Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32);
                let dest = Rectangle::new(0.0, 0.0, screen.x, screen.y);
                d.draw_texture_pro(texture, source, dest, Vector2::zero(), 0.0, Color::WHITE);
            }
        }
        None => {}
    }
}
