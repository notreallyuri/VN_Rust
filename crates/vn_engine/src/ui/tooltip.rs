use raylib::prelude::*;

use crate::ui::{self, TextStyle};
use crate::{Border, PanelStyle};
use crate::{FontRole, Fonts};

#[derive(Clone, Debug, PartialEq)]
pub struct TooltipConfig {
    pub enabled: bool,
    pub delay: f64,
    pub text: TextStyle,
    pub panel: PanelStyle,
    pub padding: f32,
    pub max_width: f32,
    pub offset: Vector2,
}

impl Default for TooltipConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            delay: 0.5,
            text: TextStyle::new(FontRole::Menu, 16.0, Color::RAYWHITE),
            panel: PanelStyle::new(Color::new(12, 12, 18, 235))
                .border(1.0, Color::new(90, 90, 110, 255)),
            padding: 8.0,
            max_width: 360.0,
            offset: Vector2::new(14.0, 20.0),
        }
    }
}

impl TooltipConfig {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn delay(mut self, seconds: f64) -> Self {
        self.delay = seconds.max(0.0);
        self
    }

    pub fn text(mut self, style: TextStyle) -> Self {
        self.text = style;
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

    pub fn border(mut self, color: Option<Color>) -> Self {
        self.panel.border = color.map(|color| Border::new(1.0, color));
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Vector2::new(x, y);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TooltipTimer {
    text: Option<String>,
    since: f64,
    dismissed: bool,
}

impl TooltipTimer {
    pub fn update(&mut self, hovered: Option<String>, now: f64, clicked: bool) {
        if hovered != self.text {
            self.text = hovered;
            self.since = now;
            self.dismissed = false;
        }
        if clicked {
            self.dismissed = true;
        }
    }

    pub fn visible(&self, now: f64, delay: f64) -> Option<&str> {
        let text = self.text.as_deref()?;
        (!self.dismissed && now - self.since >= delay).then_some(text)
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

pub fn draw_tooltip(d: &mut RaylibDrawHandle, fonts: &Fonts, text: &str, config: &TooltipConfig) {
    let style = &config.text;
    let lines = fonts.wrap(style.font, text, style.size, config.max_width);
    let line_height = style.size * 1.3;
    let width = lines
        .iter()
        .map(|line| fonts.measure(style.font, line, style.size).x)
        .fold(0.0, f32::max)
        + config.padding * 2.0;
    let height =
        lines.len() as f32 * line_height + config.padding * 2.0 - (line_height - style.size);

    let screen = ui::screen_size(d);
    let mouse = crate::frame::viewport::mouse_position(d);
    let mut x = mouse.x + config.offset.x;
    let mut y = mouse.y + config.offset.y;
    if x + width > screen.x - 4.0 {
        x = (mouse.x - config.offset.x - width).max(4.0);
    }
    if y + height > screen.y - 4.0 {
        y = (mouse.y - config.offset.y - height).max(4.0);
    }

    let panel = Rectangle::new(x, y, width, height);
    config.panel.draw(d, panel);
    for (i, line) in lines.iter().enumerate() {
        let position = Vector2::new(
            x + config.padding,
            y + config.padding + i as f32 * line_height,
        );
        ui::draw_text(d, fonts, line, position, style);
    }
}
