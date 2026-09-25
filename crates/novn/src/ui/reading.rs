use std::cell::Cell;

use crate::ui::TextStyle;

pub const SIZES: [u32; 5] = [90, 100, 115, 130, 150];

thread_local! {
    static SCALE: Cell<f32> = const { Cell::new(1.0) };
}

pub fn scale() -> f32 {
    SCALE.with(Cell::get)
}

pub fn set_percent(percent: u32) {
    let percent = percent.clamp(SIZES[0], SIZES[SIZES.len() - 1]);
    SCALE.with(|scale| scale.set(percent as f32 / 100.0));
}

pub fn text(style: &TextStyle) -> TextStyle {
    let mut style = style.clone();
    style.size *= scale();
    style
}

pub fn nearest(percent: u32) -> usize {
    SIZES
        .iter()
        .enumerate()
        .min_by_key(|(_, size)| size.abs_diff(percent))
        .map_or(1, |(index, _)| index)
}

thread_local! {
    static BACKDROP: Cell<u32> = const { Cell::new(0) };
    static OUTLINE: Cell<bool> = const { Cell::new(false) };
}

pub const OFFSETS: [(f32, f32); 8] = [
    (-1.0, -1.0),
    (0.0, -1.0),
    (1.0, -1.0),
    (-1.0, 0.0),
    (1.0, 0.0),
    (-1.0, 1.0),
    (0.0, 1.0),
    (1.0, 1.0),
];

pub fn set_backdrop(level: u32) {
    BACKDROP.with(|b| b.set(level.min(2)));
}

pub fn set_outline(on: bool) {
    OUTLINE.with(|o| o.set(on));
}

pub fn outline() -> bool {
    OUTLINE.with(Cell::get)
}

pub fn backdrop(panel: &crate::ui::shape::PanelStyle) -> crate::ui::shape::PanelStyle {
    let floor = match BACKDROP.with(Cell::get) {
        0 => return *panel,
        1 => 215,
        _ => 255,
    };
    let raise = |color: raylib::prelude::Color| raylib::prelude::Color {
        a: color.a.max(floor),
        ..color
    };
    let mut panel = *panel;
    panel.color = raise(panel.color);
    if let Some(gradient) = &mut panel.gradient {
        gradient.to = raise(gradient.to);
    }
    panel
}

pub fn outline_width(size: f32) -> f32 {
    (size / 18.0).round().max(1.0)
}

pub fn outline_color(text: raylib::prelude::Color) -> raylib::prelude::Color {
    let channel = |c: u8| c as f32 / 255.0;
    let luminance = 0.2126 * channel(text.r) + 0.7152 * channel(text.g) + 0.0722 * channel(text.b);
    let alpha = (210.0 * channel(text.a)).round() as u8;
    if luminance > 0.5 {
        raylib::prelude::Color::new(0, 0, 0, alpha)
    } else {
        raylib::prelude::Color::new(255, 255, 255, alpha)
    }
}

pub fn draw_text(
    d: &mut raylib::prelude::RaylibDrawHandle,
    fonts: &crate::ui::fonts::Fonts,
    text: &str,
    at: raylib::prelude::Vector2,
    style: &TextStyle,
) {
    if outline() {
        let mut edge = style.clone();
        edge.color = outline_color(style.color);
        let width = outline_width(style.size);
        for (dx, dy) in OFFSETS {
            let offset = raylib::prelude::Vector2::new(at.x + dx * width, at.y + dy * width);
            crate::ui::draw_text(d, fonts, text, offset, &edge);
        }
    }
    crate::ui::draw_text(d, fonts, text, at, style);
}
