use raylib::prelude::*;

use crate::ui::shape::PanelStyle;

#[derive(Clone, Debug, PartialEq)]
pub struct NamePlate {
    pub panel: PanelStyle,
    pub padding: Vector2,
    pub indent: f32,
    pub overlap: f32,
    pub min_width: f32,
}

impl Default for NamePlate {
    fn default() -> Self {
        Self {
            panel: PanelStyle::new(Color::new(0, 0, 0, 220)),
            padding: Vector2::new(18.0, 6.0),
            indent: 24.0,
            overlap: 12.0,
            min_width: 0.0,
        }
    }
}

impl NamePlate {
    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = style(self.panel);
        self
    }

    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.padding = Vector2::new(x, y);
        self
    }

    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    pub fn overlap(mut self, overlap: f32) -> Self {
        self.overlap = overlap;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    pub fn rect(&self, dialogue_box: Rectangle, name_size: Vector2) -> Rectangle {
        let width = (name_size.x + self.padding.x * 2.0).max(self.min_width);
        let height = name_size.y + self.padding.y * 2.0;
        Rectangle::new(
            dialogue_box.x + self.indent,
            dialogue_box.y - height + self.overlap,
            width,
            height,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogueBoxStyle {
    pub height: f32,
    pub margin: f32,
    pub bottom: Option<f32>,
    pub max_width: Option<f32>,
    pub padding: f32,
    pub panel: PanelStyle,
    pub name_plate: Option<NamePlate>,
}

impl Default for DialogueBoxStyle {
    fn default() -> Self {
        Self {
            height: 170.0,
            margin: 40.0,
            bottom: None,
            max_width: None,
            padding: 24.0,
            panel: PanelStyle::new(Color::new(0, 0, 0, 200)),
            name_plate: None,
        }
    }
}

impl DialogueBoxStyle {
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    pub fn bottom(mut self, distance: f32) -> Self {
        self.bottom = Some(distance);
        self
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.panel.color = color;
        self
    }

    pub fn roundness(mut self, roundness: f32) -> Self {
        self.panel = self.panel.roundness(roundness);
        self
    }

    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = style(self.panel);
        self
    }

    pub fn name_plate(mut self, plate: impl FnOnce(NamePlate) -> NamePlate) -> Self {
        self.name_plate = Some(plate(self.name_plate.unwrap_or_default()));
        self
    }

    pub fn rect(&self, screen: Vector2) -> Rectangle {
        let full = screen.x - self.margin * 2.0;
        let width = self.max_width.map_or(full, |max| max.min(full));
        Rectangle::new(
            (screen.x - width) / 2.0,
            screen.y - self.height - self.bottom.unwrap_or(self.margin),
            width,
            self.height,
        )
    }
}
