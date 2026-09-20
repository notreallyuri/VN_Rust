use raylib::prelude::*;

use crate::ui::{self, Background, ButtonStyle, TextStyle};
use crate::{Anchor, Border, FontRole, Layout, PanelStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveMenuMode {
    Save,
    Load,
}

#[derive(Clone, Debug)]
pub struct SaveMenuConfig {
    pub slots: usize,
    pub save_title: String,
    pub load_title: String,
    pub title_text: TextStyle,
    pub slot_width: f32,
    pub slot_height: f32,
    pub slot_layout: Layout,
    pub slot_panel: PanelStyle,
    pub slot_hover_color: Color,
    pub slot_hover_border: Option<Border>,
    pub slot_title_text: TextStyle,
    pub slot_summary_text: TextStyle,
    pub error_text: TextStyle,
    pub empty_label: String,
    pub back_button: ButtonStyle,
    pub back_label: String,
    pub back_keys: Vec<KeyboardKey>,
    pub confirm_overwrite: bool,
    pub confirm_load_in_game: bool,
    pub thumbnails: bool,
    pub thumbnail_color: Color,
    pub allow_delete: bool,
    pub confirm_delete: bool,
    pub delete_button: ButtonStyle,
    pub delete_label: String,
    pub delete_tooltip: Option<String>,
    pub delete_keys: Vec<KeyboardKey>,
    pub background: Option<Background>,
    pub backdrop: Color,
    pub panel: Option<PanelStyle>,
    pub panel_padding: f32,
}

impl Default for SaveMenuConfig {
    fn default() -> Self {
        Self {
            slots: 6,
            save_title: "Save Game".to_string(),
            load_title: "Load Game".to_string(),
            title_text: TextStyle::new(FontRole::Title, 44.0, Color::RAYWHITE),
            slot_width: 760.0,
            slot_height: 64.0,
            slot_layout: Layout::default().anchor(Anchor::Top).spacing(10.0),
            slot_panel: PanelStyle::new(Color::new(30, 30, 45, 230)),
            slot_hover_color: Color::new(50, 50, 72, 240),
            slot_hover_border: None,
            slot_title_text: TextStyle::new(FontRole::Menu, 20.0, Color::RAYWHITE),
            slot_summary_text: TextStyle::new(FontRole::Dialogue, 17.0, Color::LIGHTGRAY),
            error_text: TextStyle::new(FontRole::Menu, 17.0, Color::new(230, 110, 110, 255)),
            empty_label: "Empty".to_string(),
            back_button: ButtonStyle::default().size(200.0, 46.0),
            back_label: "Back".to_string(),
            back_keys: vec![KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_BACKSPACE],
            confirm_overwrite: true,
            confirm_load_in_game: true,
            thumbnails: true,
            thumbnail_color: Color::new(10, 10, 16, 255),
            allow_delete: true,
            confirm_delete: true,
            delete_button: ButtonStyle::default()
                .size(76.0, 26.0)
                .color(Color::new(90, 40, 40, 230))
                .font_size(15.0),
            delete_label: "Delete".to_string(),
            delete_tooltip: Some("Delete this save for good (Delete key)".to_string()),
            delete_keys: vec![KeyboardKey::KEY_DELETE],
            background: None,
            backdrop: Color::new(8, 8, 14, 235),
            panel: None,
            panel_padding: 40.0,
        }
    }
}

impl SaveMenuConfig {
    pub fn slots(mut self, count: usize) -> Self {
        self.slots = count.max(1);
        self
    }

    pub fn save_title(mut self, text: impl Into<String>) -> Self {
        self.save_title = text.into();
        self
    }

    pub fn load_title(mut self, text: impl Into<String>) -> Self {
        self.load_title = text.into();
        self
    }

    pub fn title_text(mut self, style: TextStyle) -> Self {
        self.title_text = style;
        self
    }

    pub fn slot_size(mut self, width: f32, height: f32) -> Self {
        self.slot_width = width;
        self.slot_height = height;
        self
    }

    pub fn slot_spacing(mut self, spacing: f32) -> Self {
        self.slot_layout = self.slot_layout.spacing(spacing);
        self
    }

    pub fn slot_layout(mut self, layout: impl FnOnce(Layout) -> Layout) -> Self {
        self.slot_layout = layout(self.slot_layout);
        self
    }

    pub fn slot_color(mut self, color: Color) -> Self {
        self.slot_panel.color = color;
        self.slot_hover_color = ui::lighten(color);
        self
    }

    pub fn slot_panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.slot_panel = style(self.slot_panel);
        self
    }

    pub fn slot_hover_color(mut self, color: Color) -> Self {
        self.slot_hover_color = color;
        self
    }

    pub fn slot_hover_border(mut self, width: f32, color: Color) -> Self {
        self.slot_hover_border = Some(Border::new(width, color));
        self
    }

    pub fn slot_hover_panel(&self) -> PanelStyle {
        PanelStyle {
            color: self.slot_hover_color,
            border: self.slot_hover_border.or(self.slot_panel.border),
            ..self.slot_panel
        }
    }

    pub fn slot_title_text(mut self, style: TextStyle) -> Self {
        self.slot_title_text = style;
        self
    }

    pub fn slot_summary_text(mut self, style: TextStyle) -> Self {
        self.slot_summary_text = style;
        self
    }

    pub fn error_text(mut self, style: TextStyle) -> Self {
        self.error_text = style;
        self
    }

    pub fn empty_label(mut self, text: impl Into<String>) -> Self {
        self.empty_label = text.into();
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

    pub fn confirm_overwrite(mut self, confirm: bool) -> Self {
        self.confirm_overwrite = confirm;
        self
    }

    pub fn confirm_load_in_game(mut self, confirm: bool) -> Self {
        self.confirm_load_in_game = confirm;
        self
    }

    pub fn thumbnails(mut self, show: bool) -> Self {
        self.thumbnails = show;
        self
    }

    pub fn thumbnail_color(mut self, color: Color) -> Self {
        self.thumbnail_color = color;
        self
    }

    pub fn allow_delete(mut self, allow: bool) -> Self {
        self.allow_delete = allow;
        self
    }

    pub fn confirm_delete(mut self, confirm: bool) -> Self {
        self.confirm_delete = confirm;
        self
    }

    pub fn delete_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.delete_button = style(self.delete_button);
        self
    }

    pub fn delete_label(mut self, text: impl Into<String>) -> Self {
        self.delete_label = text.into();
        self
    }

    pub fn delete_tooltip(mut self, text: Option<&str>) -> Self {
        self.delete_tooltip = text.map(str::to_string);
        self
    }

    pub fn delete_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.delete_keys = keys.into_iter().collect();
        self
    }

    pub fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = color;
        self
    }

    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = Some(style(self.panel.unwrap_or_default()));
        self
    }

    pub fn panel_padding(mut self, padding: f32) -> Self {
        self.panel_padding = padding;
        self
    }

    pub(super) fn thumbnail_rect(&self, slot: Rectangle) -> Option<Rectangle> {
        if !self.thumbnails {
            return None;
        }
        let height = slot.height - 12.0;
        Some(Rectangle::new(
            slot.x + 6.0,
            slot.y + 6.0,
            height * 16.0 / 9.0,
            height,
        ))
    }

    pub(super) fn delete_rect(&self, slot: Rectangle) -> Rectangle {
        let style = &self.delete_button;
        Rectangle::new(
            slot.x + slot.width - style.width - 8.0,
            slot.y + 8.0,
            style.width,
            style.height,
        )
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }
}
