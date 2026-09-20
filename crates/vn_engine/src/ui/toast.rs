use raylib::prelude::*;

use crate::ui;
use crate::ui::TextStyle;
use crate::ui::fonts::{FontRole, Fonts};
use crate::ui::shape::PanelStyle;

#[derive(Clone, Debug)]
pub struct ToastConfig {
    pub text: TextStyle,
    pub error_text: TextStyle,
    pub panel: PanelStyle,
    pub seconds: f64,
    pub margin: f32,
}

impl Default for ToastConfig {
    fn default() -> Self {
        Self {
            text: TextStyle::new(FontRole::Menu, 18.0, Color::GOLD),
            error_text: TextStyle::new(FontRole::Menu, 18.0, Color::new(230, 110, 110, 255)),
            panel: PanelStyle::new(Color::new(0, 0, 0, 200)),
            seconds: 2.5,
            margin: 16.0,
        }
    }
}

impl ToastConfig {
    pub fn text(mut self, style: TextStyle) -> Self {
        self.text = style;
        self
    }

    pub fn error_text(mut self, style: TextStyle) -> Self {
        self.error_text = style;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.panel.color = color;
        self
    }

    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = style(self.panel);
        self
    }

    pub fn seconds(mut self, seconds: f64) -> Self {
        self.seconds = seconds;
        self
    }

    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub text: String,
    pub is_error: bool,
    pub(crate) shown_at: Option<f64>,
}

impl Toast {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: false,
            shown_at: None,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: true,
            shown_at: None,
        }
    }

    pub(crate) fn expired(&self, now: f64, config: &ToastConfig) -> bool {
        self.shown_at.is_some_and(|at| now - at > config.seconds)
    }

    pub(crate) fn draw(&self, d: &mut RaylibDrawHandle, fonts: &Fonts, config: &ToastConfig) {
        let style = if self.is_error {
            &config.error_text
        } else {
            &config.text
        };
        let size = fonts.measure(style.font, &self.text, style.size);
        let panel = Rectangle::new(config.margin, config.margin, size.x + 28.0, size.y + 16.0);

        config.panel.draw(d, panel);
        ui::draw_text(
            d,
            fonts,
            &self.text,
            Vector2::new(panel.x + 14.0, panel.y + 8.0),
            style,
        );
    }
}
