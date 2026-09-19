use vn_engine::raylib::prelude::Color;
use vn_engine::{
    Background, ButtonIcon, ButtonImage, ButtonStyle, FontRole, SliderStyle, TextAlign,
    TextOverflow, TextStyle,
};

pub const INK: Color = Color::new(12, 10, 15, 255);
pub const PANEL: Color = Color::new(22, 19, 26, 238);
pub const RAISED: Color = Color::new(38, 33, 42, 240);
pub const PARCHMENT: Color = Color::new(221, 201, 164, 255);
pub const TEXT: Color = Color::new(234, 228, 216, 255);
pub const MUTED: Color = Color::new(150, 141, 128, 255);
pub const OXBLOOD: Color = Color::new(116, 38, 38, 255);
pub const BACKDROP: Color = Color::new(6, 5, 8, 205);

pub const TITLE_BACKGROUND: &str = "backgrounds/title.png";
pub const ARCHIVE_BACKGROUND: &str = "backgrounds/archive_office.png";

pub fn background(path: &str) -> Background {
    Background::Image(path.to_string())
}

pub fn heading(size: f32) -> TextStyle {
    TextStyle::new(FontRole::Title, size, PARCHMENT)
}

pub fn body(size: f32) -> TextStyle {
    TextStyle::new(FontRole::Dialogue, size, TEXT)
}

pub fn label(size: f32) -> TextStyle {
    TextStyle::new(FontRole::Menu, size, MUTED)
}

pub fn button(style: ButtonStyle) -> ButtonStyle {
    style
        .color(RAISED)
        .roundness(0.18)
        .font(FontRole::Button)
        .font_size(20.0)
        .text_color(PARCHMENT)
        .border(1.0, Color::new(110, 100, 86, 255))
        .hovered(|l| l.border(1.5, PARCHMENT).text_color(TEXT))
        .pressed(|l| l.transform(|t| t.scale(0.97)))
        .transition(0.12)
}

pub fn menu_button(style: ButtonStyle) -> ButtonStyle {
    button(style)
        .hovered(|l| l.transform(|t| t.offset(6.0, 0.0)))
        .click_sound("page_turn")
}

pub fn choice_button(style: ButtonStyle) -> ButtonStyle {
    style
        .image(ButtonImage::new("ui/choice.png").slice_all(16))
        .font(FontRole::Choice)
        .text_color(TEXT)
        .align(TextAlign::Left)
        .padding(28.0, 6.0)
        .overflow(TextOverflow::Wrap)
        .hovered(|l| {
            l.image(ButtonImage::new("ui/choice_hover.png").slice_all(16))
                .text_color(PARCHMENT)
                .transform(|t| t.offset(10.0, 0.0))
        })
        .pressed(|l| l.transform(|t| t.offset(10.0, 0.0).scale(0.98)))
        .transition(0.15)
        .click_sound("page_turn")
}

pub fn hud_button(style: ButtonStyle) -> ButtonStyle {
    button(style).shadow(0.0, 3.0, Color::new(0, 0, 0, 120))
}

pub fn icon(name: &str) -> ButtonIcon {
    ButtonIcon::new(format!("ui/{}.png", name))
        .size(16.0)
        .gap(8.0)
}

pub fn slider(style: SliderStyle) -> SliderStyle {
    style
        .track_color(RAISED)
        .fill_color(OXBLOOD)
        .knob_color(PARCHMENT)
}

pub fn danger_button(style: ButtonStyle) -> ButtonStyle {
    button(style)
        .color(OXBLOOD)
        .text_color(TEXT)
        .hovered(|l| l.transform(|t| t.skew(-8.0, 0.0)))
}
