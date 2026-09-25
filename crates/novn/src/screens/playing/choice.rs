use raylib::prelude::*;

use crate::ui::button::{ButtonIcon, ButtonImage, ButtonStyle, IconSide};
use crate::ui::layout::{Anchor, Layout};
use crate::ui::shape::PanelStyle;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChoiceImagePlace {
    #[default]
    Fill,
    Icon,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChoiceImageStyle {
    pub place: ChoiceImagePlace,
    pub tint: Color,
    pub size: f32,
    pub side: IconSide,
    pub gap: f32,
}

impl Default for ChoiceImageStyle {
    fn default() -> Self {
        Self {
            place: ChoiceImagePlace::Fill,
            tint: Color::WHITE,
            size: 32.0,
            side: IconSide::Left,
            gap: 12.0,
        }
    }
}

impl ChoiceImageStyle {
    pub fn fill(mut self) -> Self {
        self.place = ChoiceImagePlace::Fill;
        self
    }

    pub fn icon(mut self) -> Self {
        self.place = ChoiceImagePlace::Icon;
        self
    }

    pub fn tint(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(0.0);
        self
    }

    pub fn side(mut self, side: IconSide) -> Self {
        self.side = side;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn apply(&self, style: ButtonStyle, path: String) -> ButtonStyle {
        match self.place {
            ChoiceImagePlace::Fill => style.image(ButtonImage::new(path).tint(self.tint)),
            ChoiceImagePlace::Icon => style.icon(
                ButtonIcon::new(path)
                    .size(self.size)
                    .side(self.side)
                    .gap(self.gap)
                    .tint(self.tint),
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChoicePreviewStyle {
    pub size: Vector2,
    pub anchor: Anchor,
    pub margin: f32,
    pub padding: f32,
    pub panel: PanelStyle,
}

impl Default for ChoicePreviewStyle {
    fn default() -> Self {
        Self {
            size: Vector2::new(360.0, 240.0),
            anchor: Anchor::TopRight,
            margin: 40.0,
            padding: 10.0,
            panel: PanelStyle::new(Color::new(0, 0, 0, 200)),
        }
    }
}

impl ChoicePreviewStyle {
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vector2::new(width.max(0.0), height.max(0.0));
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = style(self.panel);
        self
    }

    pub fn rect(&self, screen: Vector2) -> Rectangle {
        let area = Rectangle::new(
            self.margin,
            self.margin,
            (screen.x - self.margin * 2.0).max(0.0),
            (screen.y - self.margin * 2.0).max(0.0),
        );
        Layout::default()
            .anchor(self.anchor)
            .place(area, &[self.size])
            .into_iter()
            .next()
            .unwrap_or(area)
    }

    pub fn picture_rect(&self, panel: Rectangle, natural: Vector2) -> Rectangle {
        let inner = Rectangle::new(
            panel.x + self.padding,
            panel.y + self.padding,
            (panel.width - self.padding * 2.0).max(0.0),
            (panel.height - self.padding * 2.0).max(0.0),
        );
        if natural.x <= 0.0 || natural.y <= 0.0 {
            return inner;
        }

        let scale = (inner.width / natural.x).min(inner.height / natural.y);
        let width = natural.x * scale;
        let height = natural.y * scale;
        Rectangle::new(
            inner.x + (inner.width - width) / 2.0,
            inner.y + (inner.height - height) / 2.0,
            width,
            height,
        )
    }
}
