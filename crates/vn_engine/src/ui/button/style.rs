use std::fmt;
use std::rc::Rc;

use raylib::prelude::*;

use super::{
    Border, ButtonIcon, ButtonImage, Look, Shadow, StateAmounts, TextAlign, TextOverflow, Transform,
};
use crate::ui::fonts::FontRole;
use crate::ui::shape::Corners;
use crate::ui::{TextStyle, lighten};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ButtonLook {
    pub fill: Option<Color>,
    pub text_color: Option<Color>,
    pub border: Option<Border>,
    pub shadow: Option<Shadow>,
    pub image: Option<ButtonImage>,
    pub transform: Option<Transform>,
    pub opacity: Option<f32>,
    pub underline: Option<Border>,
}

impl ButtonLook {
    pub fn underline(mut self, width: f32, color: Color) -> Self {
        self.underline = Some(Border::new(width, color));
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        self
    }

    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.border = Some(Border::new(width, color));
        self
    }

    pub fn shadow(mut self, x: f32, y: f32, color: Color) -> Self {
        self.shadow = Some(Shadow::new(x, y, color));
        self
    }

    pub fn image(mut self, image: ButtonImage) -> Self {
        self.image = Some(image);
        self
    }

    pub fn transform(mut self, transform: impl FnOnce(Transform) -> Transform) -> Self {
        self.transform = Some(transform(self.transform.unwrap_or_default()));
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }
}

#[derive(Clone)]
pub struct StyleOverride(Rc<dyn Fn(ButtonStyle) -> ButtonStyle>);

impl StyleOverride {
    pub fn new(style: impl Fn(ButtonStyle) -> ButtonStyle + 'static) -> Self {
        Self(Rc::new(style))
    }

    pub fn apply(&self, base: &ButtonStyle) -> ButtonStyle {
        (self.0)(base.clone())
    }
}

impl fmt::Debug for StyleOverride {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StyleOverride(..)")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonStyle {
    pub width: f32,
    pub height: f32,
    pub color: Color,
    pub corners: Corners,
    pub text: TextStyle,
    pub align: TextAlign,
    pub padding: Vector2,
    pub overflow: TextOverflow,
    pub border: Option<Border>,
    pub shadow: Option<Shadow>,
    pub image: Option<ButtonImage>,
    pub icon: Option<ButtonIcon>,
    pub transform: Transform,
    pub hovered: ButtonLook,
    pub pressed: ButtonLook,
    pub focused: ButtonLook,
    pub disabled: ButtonLook,
    pub transition: f32,
    pub hover_sound: Option<String>,
    pub click_sound: Option<String>,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        let color = Color::new(40, 40, 60, 255);
        Self {
            width: 240.0,
            height: 52.0,
            color,
            corners: Corners::SQUARE,
            text: TextStyle::new(FontRole::Button, 22.0, Color::WHITE),
            align: TextAlign::Center,
            padding: Vector2::new(12.0, 6.0),
            overflow: TextOverflow::Ellipsis,
            border: None,
            shadow: None,
            image: None,
            icon: None,
            transform: Transform::IDENTITY,
            hovered: ButtonLook::default().fill(lighten(color)),
            pressed: ButtonLook::default().fill(color),
            focused: ButtonLook::default().border(2.0, Color::new(255, 255, 255, 200)),
            disabled: ButtonLook::default().opacity(0.45),
            transition: 0.0,
            hover_sound: None,
            click_sound: None,
        }
    }
}

impl ButtonStyle {
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self.hovered.fill = Some(lighten(color));
        self.pressed.fill = Some(color);
        self
    }

    pub fn hover_color(mut self, color: Color) -> Self {
        self.hovered.fill = Some(color);
        self
    }

    pub fn pressed_color(mut self, color: Color) -> Self {
        self.pressed.fill = Some(color);
        self
    }

    pub fn roundness(self, roundness: f32) -> Self {
        self.corners(Corners::roundness(roundness))
    }

    pub fn corners(mut self, corners: impl Into<Corners>) -> Self {
        self.corners = corners.into();
        self
    }

    pub fn text(mut self, text: TextStyle) -> Self {
        self.text = text;
        self
    }

    pub fn font(mut self, font: FontRole) -> Self {
        self.text.font = font;
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.text.size = size;
        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text.color = color;
        self
    }

    pub fn hover_text_color(mut self, color: Color) -> Self {
        self.hovered.text_color = Some(color);
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.padding = Vector2::new(x, y);
        self
    }

    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.border = Some(Border::new(width, color));
        self
    }

    pub fn no_border(mut self) -> Self {
        self.border = None;
        self
    }

    pub fn shadow(mut self, x: f32, y: f32, color: Color) -> Self {
        self.shadow = Some(Shadow::new(x, y, color));
        self
    }

    pub fn image(mut self, image: ButtonImage) -> Self {
        self.image = Some(image);
        self
    }

    pub fn icon(mut self, icon: ButtonIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn transform(mut self, transform: impl FnOnce(Transform) -> Transform) -> Self {
        self.transform = transform(self.transform);
        self
    }

    pub fn skew(self, x_degrees: f32, y_degrees: f32) -> Self {
        self.transform(|t| t.skew(x_degrees, y_degrees))
    }

    pub fn rotate(self, degrees: f32) -> Self {
        self.transform(|t| t.rotate(degrees))
    }

    pub fn scale(self, factor: f32) -> Self {
        self.transform(|t| t.scale(factor))
    }

    pub fn hovered(mut self, look: impl FnOnce(ButtonLook) -> ButtonLook) -> Self {
        self.hovered = look(self.hovered);
        self
    }

    pub fn pressed(mut self, look: impl FnOnce(ButtonLook) -> ButtonLook) -> Self {
        self.pressed = look(self.pressed);
        self
    }

    pub fn focused(mut self, look: impl FnOnce(ButtonLook) -> ButtonLook) -> Self {
        self.focused = look(self.focused);
        self
    }

    pub fn disabled(mut self, look: impl FnOnce(ButtonLook) -> ButtonLook) -> Self {
        self.disabled = look(self.disabled);
        self
    }

    pub fn transition(mut self, seconds: f32) -> Self {
        self.transition = seconds.max(0.0);
        self
    }

    pub fn hover_sound(mut self, id: impl Into<String>) -> Self {
        self.hover_sound = Some(id.into());
        self
    }

    pub fn click_sound(mut self, id: impl Into<String>) -> Self {
        self.click_sound = Some(id.into());
        self
    }

    pub fn contains(&self, rect: Rectangle, point: Vector2) -> bool {
        self.transform.contains(rect, point)
    }

    pub fn look(&self, amounts: StateAmounts) -> Look {
        let mut look = Look {
            fill: self.color,
            text_color: self.text.color,
            border: self.border,
            shadow: self.shadow,
            image: self.image.clone(),
            transform: self.transform,
            opacity: 1.0,
            underline: None,
        };
        look.blend(&self.focused, amounts.focus);
        look.blend(&self.hovered, amounts.hover);
        look.blend(&self.pressed, amounts.press);
        if amounts.disabled {
            look.blend(&self.disabled, 1.0);
        }
        look
    }

    pub fn image_paths(&self) -> Vec<&str> {
        let looks = [&self.hovered, &self.pressed, &self.focused, &self.disabled];
        self.image
            .iter()
            .chain(looks.iter().filter_map(|look| look.image.as_ref()))
            .map(|image| image.path.as_str())
            .chain(self.icon.iter().map(|icon| icon.path.as_str()))
            .collect()
    }
}
