use raylib::prelude::*;

use super::{Border, ButtonImage, ButtonLook, Shadow, Transform};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StateAmounts {
    pub hover: f32,
    pub press: f32,
    pub focus: f32,
    pub disabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    pub fill: Color,
    pub text_color: Color,
    pub border: Option<Border>,
    pub shadow: Option<Shadow>,
    pub image: Option<ButtonImage>,
    pub transform: Transform,
    pub opacity: f32,
    pub underline: Option<Border>,
}

impl Look {
    pub(crate) fn blend(&mut self, other: &ButtonLook, t: f32) {
        if t <= 0.0 {
            return;
        }
        let t = t.min(1.0);

        if let Some(fill) = other.fill {
            self.fill = mix_color(self.fill, fill, t);
        }
        if let Some(color) = other.text_color {
            self.text_color = mix_color(self.text_color, color, t);
        }
        if let Some(border) = other.border {
            let from = self.border.unwrap_or(Border::new(0.0, border.color));
            self.border = Some(Border::new(
                mix(from.width, border.width, t),
                mix_color(from.color, border.color, t),
            ));
        }
        if let Some(shadow) = other.shadow {
            let from = self.shadow.unwrap_or(Shadow {
                offset: Vector2::zero(),
                color: Color::new(shadow.color.r, shadow.color.g, shadow.color.b, 0),
            });
            self.shadow = Some(Shadow {
                offset: Vector2::new(
                    mix(from.offset.x, shadow.offset.x, t),
                    mix(from.offset.y, shadow.offset.y, t),
                ),
                color: mix_color(from.color, shadow.color, t),
            });
        }
        if let Some(image) = &other.image
            && t >= 0.5
        {
            self.image = Some(image.clone());
        }
        if let Some(transform) = &other.transform {
            self.transform = self.transform.lerp(transform, t);
        }
        if let Some(opacity) = other.opacity {
            self.opacity = mix(self.opacity, opacity, t);
        }
        if let Some(line) = other.underline {
            let from = self.underline.unwrap_or(Border::new(
                line.width,
                Color::new(line.color.r, line.color.g, line.color.b, 0),
            ));
            self.underline = Some(Border::new(
                mix(from.width, line.width, t),
                mix_color(from.color, line.color, t),
            ));
        }
    }
}

pub(crate) fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn mix_color(a: Color, b: Color, t: f32) -> Color {
    let channel = |x: u8, y: u8| mix(x as f32, y as f32, t).round().clamp(0.0, 255.0) as u8;
    Color::new(
        channel(a.r, b.r),
        channel(a.g, b.g),
        channel(a.b, b.b),
        channel(a.a, b.a),
    )
}

pub(crate) fn faded(color: Color, opacity: f32) -> Color {
    Color::new(
        color.r,
        color.g,
        color.b,
        (color.a as f32 * opacity).round() as u8,
    )
}
