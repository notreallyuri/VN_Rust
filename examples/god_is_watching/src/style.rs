use vn_engine::frame::scenery::{Scenery, Weather};
use vn_engine::prelude::*;
use vn_engine::raylib::prelude::Color;
use vn_engine::ui::prelude::*;

pub const INK: Color = Color::new(12, 10, 8, 255);
pub const PANEL: Color = Color::new(20, 16, 12, 240);
pub const PANEL_DEEP: Color = Color::new(14, 11, 9, 250);
pub const RAISED: Color = Color::new(40, 32, 24, 235);
pub const RAISED_HOVER: Color = Color::new(60, 47, 34, 245);
pub const BRASS: Color = Color::new(190, 156, 100, 255);
pub const BRASS_DIM: Color = Color::new(124, 100, 66, 255);
pub const PARCHMENT: Color = Color::new(226, 206, 168, 255);
pub const TEXT: Color = Color::new(236, 228, 212, 255);
pub const MUTED: Color = Color::new(160, 146, 124, 255);
pub const OXBLOOD: Color = Color::new(88, 28, 24, 240);
pub const OXBLOOD_HOVER: Color = Color::new(116, 36, 30, 248);
pub const BACKDROP: Color = Color::new(8, 6, 5, 200);

pub const FRAME_CORNER: f32 = 14.0;
pub const CUT: f32 = 8.0;

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

pub fn section(size: f32) -> TextStyle {
    TextStyle::new(FontRole::Menu, size, BRASS)
}

pub fn frame(panel: PanelStyle) -> PanelStyle {
    panel
        .color(PANEL)
        .corners(Corners::scoop(FRAME_CORNER))
        .border(1.0, BRASS_DIM)
        .inner_border(5.0, 1.0, BRASS_DIM.alpha(0.35))
        .shadow(0.0, 8.0, Color::new(0, 0, 0, 110))
}

pub fn inset(panel: PanelStyle) -> PanelStyle {
    panel
        .color(Color::new(8, 6, 5, 150))
        .corners(Corners::bevel(CUT))
        .border(1.0, BRASS_DIM.alpha(0.45))
}

pub fn plate(panel: PanelStyle) -> PanelStyle {
    panel
        .color(PANEL_DEEP)
        .corners(Corners::bevel(6.0))
        .border(1.0, BRASS_DIM)
}

pub const TITLE_CARD_Y: f32 = 0.44;

pub fn title_card() -> TextStyle {
    TextStyle::new(FontRole::Title, 66.0, PARCHMENT).spacing(12.0)
}

pub fn title_card_subtitle() -> TextStyle {
    label(17.0).spacing(3.0)
}

pub fn scenery(scenery: Scenery) -> Scenery {
    scenery
        .motion(1.07, 48.0)
        .pan(0.35, -0.25)
        .weather(Weather::dust(70))
        .vignette(Color::new(4, 3, 2, 200), 0.24)
        .letterbox(|l| l.height(84.0).color(INK).rule(1.0, BRASS_DIM.alpha(0.6)))
}

pub fn menu_link(style: ButtonStyle) -> ButtonStyle {
    style
        .color(Color::new(0, 0, 0, 0))
        .text(TextStyle::new(FontRole::Menu, 15.0, MUTED).spacing(3.0))
        .hovered(|l| l.text_color(PARCHMENT).underline(1.0, BRASS))
        .focused(|l| {
            l.text_color(PARCHMENT)
                .underline(1.0, PARCHMENT)
                .border(0.0, Color::new(0, 0, 0, 0))
        })
        .pressed(|l| l.text_color(TEXT).underline(1.0, BRASS))
        .disabled(|l| l.opacity(0.35))
        .transition(0.18)
        .click_sound("page_turn")
}

pub fn link_width(label: &str) -> f32 {
    label.chars().count() as f32 * 12.5 + 24.0
}

pub fn menu_link_danger(style: ButtonStyle) -> ButtonStyle {
    menu_link(style)
        .hovered(|l| {
            l.text_color(Color::new(214, 120, 96, 255))
                .underline(1.0, Color::new(170, 70, 56, 255))
        })
        .focused(|l| {
            l.text_color(Color::new(214, 120, 96, 255))
                .underline(1.0, Color::new(214, 120, 96, 255))
        })
}

pub fn button(style: ButtonStyle) -> ButtonStyle {
    style
        .color(RAISED)
        .corners(Corners::bevel(CUT))
        .font(FontRole::Button)
        .font_size(19.0)
        .text_color(PARCHMENT)
        .border(1.0, BRASS_DIM)
        .hovered(|l| l.fill(RAISED_HOVER).border(1.0, BRASS).text_color(TEXT))
        .focused(|l| l.fill(RAISED_HOVER).border(1.5, PARCHMENT).text_color(TEXT))
        .pressed(|l| l.fill(RAISED).border(1.0, BRASS))
        .transition(0.12)
}

pub fn danger_button(style: ButtonStyle) -> ButtonStyle {
    button(style)
        .color(OXBLOOD)
        .text_color(TEXT)
        .border(1.0, Color::new(150, 78, 60, 255))
        .hovered(|l| l.fill(OXBLOOD_HOVER).border(1.0, PARCHMENT))
        .focused(|l| l.fill(OXBLOOD_HOVER).border(1.5, PARCHMENT))
        .pressed(|l| l.fill(OXBLOOD).border(1.0, PARCHMENT))
}

pub fn choice_button(style: ButtonStyle) -> ButtonStyle {
    let frame = |name: &str| ButtonImage::new(format!("ui/{}.png", name)).slice_all(16);
    style
        .image(frame("choice"))
        .font(FontRole::Choice)
        .text_color(TEXT)
        .align(TextAlign::Left)
        .padding(30.0, 6.0)
        .overflow(TextOverflow::Wrap)
        .hovered(|l| l.image(frame("choice_hover")).text_color(PARCHMENT))
        .focused(|l| {
            l.image(frame("choice_hover"))
                .border(0.0, PARCHMENT)
                .text_color(PARCHMENT)
        })
        .pressed(|l| l.image(frame("choice_hover")).text_color(TEXT))
        .transition(0.12)
        .click_sound("page_turn")
}

pub fn hud_button(style: ButtonStyle) -> ButtonStyle {
    style
        .color(Color::new(0, 0, 0, 0))
        .corners(Corners::bevel(6.0))
        .font(FontRole::Menu)
        .font_size(16.0)
        .text_color(Color::new(196, 178, 146, 255))
        .hovered(|l| {
            l.fill(RAISED.alpha(0.8))
                .border(1.0, BRASS_DIM)
                .text_color(PARCHMENT)
        })
        .focused(|l| {
            l.fill(RAISED.alpha(0.8))
                .border(1.0, PARCHMENT)
                .text_color(PARCHMENT)
        })
        .pressed(|l| l.fill(RAISED).text_color(TEXT))
        .transition(0.12)
}

pub fn hud_chip(style: ButtonStyle) -> ButtonStyle {
    hud_button(style)
        .color(PANEL.alpha(0.72))
        .border(1.0, BRASS_DIM.alpha(0.55))
        .hovered(|l| l.fill(RAISED).border(1.0, BRASS).text_color(PARCHMENT))
}

pub fn icon(name: &str) -> ButtonIcon {
    ButtonIcon::new(format!("ui/{}.png", name))
        .size(14.0)
        .gap(7.0)
}

pub fn slider(style: SliderStyle) -> SliderStyle {
    style
        .track_color(RAISED)
        .fill_color(BRASS_DIM)
        .knob_color(PARCHMENT)
}
