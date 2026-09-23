use raylib::prelude::*;

use crate::ui::button::ButtonStyle;
use crate::ui::fonts::FontRole;
use crate::ui::shape::PanelStyle;
use crate::ui::{Background, TextStyle};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextRole {
    pub font: FontRole,
    pub color: Color,
}

impl TextRole {
    pub fn new(font: FontRole, color: Color) -> Self {
        Self { font, color }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Theme {
    pub panel: Option<PanelStyle>,
    pub inset: Option<PanelStyle>,
    pub plate: Option<PanelStyle>,
    pub backdrop: Option<Color>,
    pub button: Option<ButtonStyle>,
    pub danger: Option<ButtonStyle>,
    pub title: Option<TextRole>,
    pub section: Option<TextRole>,
    pub body: Option<TextRole>,
    pub label: Option<TextRole>,
    pub background: Option<Background>,
}

impl Theme {
    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = Some(style(PanelStyle::default()));
        self
    }

    pub fn inset(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.inset = Some(style(PanelStyle::default()));
        self
    }

    pub fn plate(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.plate = Some(style(PanelStyle::default()));
        self
    }

    pub fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = Some(color);
        self
    }

    pub fn button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.button = Some(style(ButtonStyle::default()));
        self
    }

    pub fn danger(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.danger = Some(style(ButtonStyle::default()));
        self
    }

    pub fn title(mut self, font: FontRole, color: Color) -> Self {
        self.title = Some(TextRole::new(font, color));
        self
    }

    pub fn section(mut self, font: FontRole, color: Color) -> Self {
        self.section = Some(TextRole::new(font, color));
        self
    }

    pub fn body(mut self, font: FontRole, color: Color) -> Self {
        self.body = Some(TextRole::new(font, color));
        self
    }

    pub fn label(mut self, font: FontRole, color: Color) -> Self {
        self.label = Some(TextRole::new(font, color));
        self
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    pub fn text(&self, role: &Option<TextRole>, current: TextStyle) -> TextStyle {
        match role {
            Some(role) => current.themed(*role),
            None => current,
        }
    }

    pub fn buttons(&self, role: &Option<ButtonStyle>, current: ButtonStyle) -> ButtonStyle {
        match role {
            Some(role) => current.themed(role),
            None => current,
        }
    }

    pub fn surface(&self, role: &Option<PanelStyle>, current: PanelStyle) -> PanelStyle {
        role.unwrap_or(current)
    }

    pub fn dim(&self, current: Color) -> Color {
        self.backdrop.unwrap_or(current)
    }

    pub fn behind(&self, current: Option<Background>) -> Option<Background> {
        self.background.clone().or(current)
    }
}
