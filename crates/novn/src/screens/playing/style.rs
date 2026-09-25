use std::fmt;
use std::rc::Rc;

use raylib::prelude::*;

use crate::ui::shape::PanelStyle;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BustSide {
    #[default]
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BustStyle {
    pub width: f32,
    pub side: BustSide,
    pub gap: f32,
    pub rise: f32,
    pub sink: f32,
}

impl Default for BustStyle {
    fn default() -> Self {
        Self {
            width: 150.0,
            side: BustSide::Left,
            gap: 16.0,
            rise: 0.0,
            sink: 0.0,
        }
    }
}

impl BustStyle {
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(0.0);
        self
    }

    pub fn side(mut self, side: BustSide) -> Self {
        self.side = side;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn rise(mut self, rise: f32) -> Self {
        self.rise = rise.max(0.0);
        self
    }

    pub fn sink(mut self, sink: f32) -> Self {
        self.sink = sink.max(0.0);
        self
    }

    pub fn rect(&self, box_rect: Rectangle, natural: Vector2) -> Rectangle {
        let height = box_rect.height + self.rise + self.sink;
        let width = match natural.y > 0.0 && natural.x > 0.0 {
            true => (natural.x / natural.y) * height,
            false => self.width,
        };
        let width = width.min(self.width);
        let height = match natural.x > 0.0 && natural.y > 0.0 {
            true => (natural.y / natural.x) * width,
            false => height,
        };
        let x = match self.side {
            BustSide::Left => box_rect.x + self.gap,
            BustSide::Right => box_rect.x + box_rect.width - self.gap - width,
        };
        Rectangle::new(
            x,
            box_rect.y + box_rect.height + self.sink - height,
            width,
            height,
        )
    }

    pub fn reserved(&self) -> f32 {
        self.width + self.gap * 2.0
    }
}

#[derive(Clone)]
pub struct BoxOverride(Rc<dyn Fn(DialogueBoxStyle) -> DialogueBoxStyle>);

impl BoxOverride {
    pub fn new(style: impl Fn(DialogueBoxStyle) -> DialogueBoxStyle + 'static) -> Self {
        Self(Rc::new(style))
    }

    pub fn apply(&self, base: &DialogueBoxStyle) -> DialogueBoxStyle {
        (self.0)(base.clone())
    }
}

impl fmt::Debug for BoxOverride {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BoxOverride(..)")
    }
}

impl PartialEq for BoxOverride {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

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
    pub bust: Option<BustStyle>,
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
            bust: None,
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

    pub fn bust(mut self, style: impl FnOnce(BustStyle) -> BustStyle) -> Self {
        self.bust = Some(style(self.bust.unwrap_or_default()));
        self
    }

    pub fn text_area(&self, box_rect: Rectangle) -> Rectangle {
        let reserved = self.bust.as_ref().map_or(0.0, BustStyle::reserved);
        let left = match self.bust.as_ref().map(|b| b.side) {
            Some(BustSide::Left) => reserved,
            _ => 0.0,
        };
        Rectangle::new(
            box_rect.x + self.padding + left,
            box_rect.y + self.padding,
            (box_rect.width - self.padding * 2.0 - reserved).max(0.0),
            (box_rect.height - self.padding * 2.0).max(0.0),
        )
    }

    pub fn rect(&self, screen: Vector2) -> Rectangle {
        let full = screen.x - self.margin * 2.0;
        let width = self.max_width.map_or(full, |max| max.min(full));
        let inside = (self.height - self.padding * 2.0).max(0.0);
        let height = self.height + inside * (crate::ui::reading::scale() - 1.0);
        Rectangle::new(
            (screen.x - width) / 2.0,
            screen.y - height - self.bottom.unwrap_or(self.margin),
            width,
            height,
        )
    }
}
