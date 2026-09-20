use raylib::prelude::*;

pub use crate::button::*;
use crate::{FontRole, Fonts, GameContext, ResourceManager};

#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub font: FontRole,
    pub size: f32,
    pub color: Color,
    pub spacing: f32,
}

impl TextStyle {
    pub fn new(font: FontRole, size: f32, color: Color) -> Self {
        Self {
            font,
            size,
            color,
            spacing: 0.0,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
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
pub struct SliderStyle {
    pub track_height: f32,
    pub knob_radius: f32,
    pub track_color: Color,
    pub fill_color: Color,
    pub knob_color: Color,
    pub knob_hover_color: Color,
    pub step_marks: bool,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            track_height: 6.0,
            knob_radius: 10.0,
            track_color: Color::new(60, 60, 80, 255),
            fill_color: Color::new(150, 150, 200, 255),
            knob_color: Color::new(225, 225, 235, 255),
            knob_hover_color: Color::WHITE,
            step_marks: true,
        }
    }
}

impl SliderStyle {
    pub fn track_height(mut self, height: f32) -> Self {
        self.track_height = height;
        self
    }

    pub fn knob_radius(mut self, radius: f32) -> Self {
        self.knob_radius = radius;
        self
    }

    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = color;
        self
    }

    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    pub fn knob_color(mut self, color: Color) -> Self {
        self.knob_color = color;
        self.knob_hover_color = lighten(color);
        self
    }

    pub fn step_marks(mut self, show: bool) -> Self {
        self.step_marks = show;
        self
    }
}

pub fn slider_fraction(area: Rectangle, x: f32) -> f32 {
    if area.width <= 0.0 {
        return 0.0;
    }
    ((x - area.x) / area.width).clamp(0.0, 1.0)
}

pub fn slider_step(fraction: f32, steps: usize) -> usize {
    if steps <= 1 {
        return 0;
    }
    (fraction.clamp(0.0, 1.0) * (steps - 1) as f32).round() as usize
}

pub fn draw_slider(
    d: &mut RaylibDrawHandle,
    area: Rectangle,
    fraction: f32,
    steps: Option<usize>,
    highlighted: bool,
    style: &SliderStyle,
) {
    let fraction = fraction.clamp(0.0, 1.0);
    let center_y = area.y + area.height / 2.0;
    let track = Rectangle::new(
        area.x,
        center_y - style.track_height / 2.0,
        area.width,
        style.track_height,
    );
    let filled = Rectangle::new(track.x, track.y, track.width * fraction, track.height);

    d.draw_rectangle_rounded(track, 1.0, 6, style.track_color);
    if filled.width > 0.0 {
        d.draw_rectangle_rounded(filled, 1.0, 6, style.fill_color);
    }

    if style.step_marks
        && let Some(steps) = steps.filter(|&n| n > 1 && n <= 12)
    {
        for i in 0..steps {
            let x = area.x + area.width * i as f32 / (steps - 1) as f32;
            let color = if (i as f32) / ((steps - 1) as f32) <= fraction {
                style.fill_color
            } else {
                style.track_color
            };
            d.draw_circle_v(Vector2::new(x, center_y), style.track_height * 0.9, color);
        }
    }

    let knob = Vector2::new(area.x + area.width * fraction, center_y);
    let color = if highlighted {
        style.knob_hover_color
    } else {
        style.knob_color
    };
    d.draw_circle_v(knob, style.knob_radius, color);
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
    rect.check_collision_point_rec(crate::viewport::mouse_position(rl))
}

pub fn is_clicked(rl: &RaylibHandle, rect: Rectangle) -> bool {
    is_hovered(rl, rect) && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
}

pub fn draw_text(
    d: &mut RaylibDrawHandle,
    fonts: &Fonts,
    text: &str,
    position: Vector2,
    style: &TextStyle,
) {
    fonts.draw_spaced(
        d,
        style.font,
        text,
        position,
        style.size,
        style.spacing,
        style.color,
    );
}

pub fn measure_text(fonts: &Fonts, text: &str, style: &TextStyle) -> Vector2 {
    fonts.measure_spaced(style.font, text, style.size, style.spacing)
}

pub fn draw_text_centered(
    d: &mut RaylibDrawHandle,
    fonts: &Fonts,
    text: &str,
    center: Vector2,
    style: &TextStyle,
) {
    let size = measure_text(fonts, text, style);
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

pub fn fit_text(fonts: &Fonts, style: &TextStyle, text: &str, max_width: f32) -> String {
    let fits = |candidate: &str| measure_text(fonts, candidate, style).x <= max_width;
    if fits(text) {
        return text.to_string();
    }

    let chars: Vec<char> = text.chars().collect();
    let (mut low, mut high) = (0, chars.len());
    while low < high {
        let mid = (low + high).div_ceil(2);
        let candidate: String = chars[..mid].iter().collect();
        if fits(&format!("{}…", candidate.trim_end())) {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    let kept: String = chars[..low].iter().collect();
    format!("{}…", kept.trim_end())
}

pub fn screen_size(rl: &RaylibHandle) -> Vector2 {
    crate::viewport::size(rl)
}

pub fn load_background(ctx: &mut GameContext, background: Option<&Background>) {
    if let Some(Background::Image(path)) = background {
        ctx.resources.get_or_load(path, ctx.rl, ctx.thread);
    }
}

pub fn cover_rect(size: Vector2, area: Rectangle) -> Rectangle {
    if size.x <= 0.0 || size.y <= 0.0 {
        return area;
    }
    let scale = (area.width / size.x).max(area.height / size.y);
    let (width, height) = (size.x * scale, size.y * scale);
    Rectangle::new(
        area.x + (area.width - width) / 2.0,
        area.y + (area.height - height) / 2.0,
        width,
        height,
    )
}

pub fn draw_texture_cover(d: &mut RaylibDrawHandle, texture: &Texture2D, area: Rectangle) {
    let (w, h) = (texture.width as f32, texture.height as f32);
    let scale = (area.width / w).max(area.height / h);
    let (source_w, source_h) = (area.width / scale, area.height / scale);
    let source = Rectangle::new(
        (w - source_w) / 2.0,
        (h - source_h) / 2.0,
        source_w,
        source_h,
    );
    d.draw_texture_pro(texture, source, area, Vector2::zero(), 0.0, Color::WHITE);
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
                draw_texture_cover(d, texture, Rectangle::new(0.0, 0.0, screen.x, screen.y));
            }
        }
        None => {}
    }
}
