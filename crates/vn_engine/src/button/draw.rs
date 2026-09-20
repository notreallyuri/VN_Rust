use raylib::ffi;
use raylib::prelude::*;

use super::anim::ANIMATIONS;
use super::{
    Anim, ButtonImage, ButtonStyle, IconSide, Look, StateAmounts, TextAlign, TextOverflow,
    Transform, button_key, faded, step_amount,
};
use crate::DrawContext;
use crate::shape::{self, Corners};
use crate::ui::{TextStyle, fit_text};

pub fn draw_button(
    d: &mut RaylibDrawHandle,
    ctx: &DrawContext,
    rect: Rectangle,
    label: &str,
    style: &ButtonStyle,
) {
    Button::new(label, style).draw(d, ctx, rect);
}

pub struct Button<'a> {
    label: &'a str,
    style: &'a ButtonStyle,
    disabled: bool,
    focused: bool,
    active: bool,
    opacity: f32,
}

impl<'a> Button<'a> {
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn new(label: &'a str, style: &'a ButtonStyle) -> Self {
        Self {
            label,
            style,
            disabled: false,
            focused: false,
            active: false,
            opacity: 1.0,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    fn amounts(&self, d: &RaylibDrawHandle, ctx: &DrawContext, rect: Rectangle) -> StateAmounts {
        let style = self.style;
        let hovered = ctx.interactive
            && !self.disabled
            && style.contains(rect, crate::viewport::mouse_position(d));
        let held = d.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
        let target = StateAmounts {
            hover: f32::from(u8::from(hovered)),
            press: f32::from(u8::from(
                (hovered && held) || (self.active && !self.disabled),
            )),
            focus: f32::from(u8::from(self.focused && !self.disabled)),
            disabled: self.disabled,
        };

        let now = d.get_time();
        let dt = d.get_frame_time();
        let key = button_key(rect, self.label);
        ANIMATIONS.with(|cell| {
            let (last_sweep, anims) = &mut *cell.borrow_mut();
            if now - *last_sweep > 1.0 {
                anims.retain(|_, anim| now - anim.seen < 1.0);
                *last_sweep = now;
            }

            let anim = anims.entry(key).or_insert(Anim {
                amounts: target,
                seen: now,
            });
            let seconds = style.transition;
            anim.amounts = StateAmounts {
                hover: step_amount(anim.amounts.hover, target.hover, dt, seconds),
                press: step_amount(anim.amounts.press, target.press, dt, seconds * 0.5),
                focus: step_amount(anim.amounts.focus, target.focus, dt, seconds),
                disabled: target.disabled,
            };
            anim.seen = now;
            anim.amounts
        })
    }

    pub fn draw(self, d: &mut RaylibDrawHandle, ctx: &DrawContext, rect: Rectangle) {
        let mut look = self.style.look(self.amounts(d, ctx, rect));
        look.opacity *= self.opacity;
        if look.opacity <= 0.0 {
            return;
        }
        with_transform(rect, &look.transform, || {
            draw_body(d, ctx, rect, self.style, &look);
            draw_content(d, ctx, rect, self.label, self.style, &look);
        });
    }
}

fn with_transform(rect: Rectangle, transform: &Transform, draw: impl FnOnce()) {
    if transform.is_identity() {
        draw();
        return;
    }

    let pivot = transform.pivot(rect);
    let (sx, sy) = transform.shear();
    let shear = [
        1.0, sy, 0.0, 0.0, //
        sx, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ];

    unsafe {
        ffi::rlDrawRenderBatchActive();
        ffi::rlPushMatrix();
        ffi::rlTranslatef(
            pivot.x + transform.offset.x,
            pivot.y + transform.offset.y,
            0.0,
        );
        ffi::rlRotatef(transform.rotation, 0.0, 0.0, 1.0);
        ffi::rlMultMatrixf(shear.as_ptr());
        ffi::rlScalef(transform.scale.x, transform.scale.y, 1.0);
        ffi::rlTranslatef(-pivot.x, -pivot.y, 0.0);
    }
    draw();
    unsafe {
        ffi::rlDrawRenderBatchActive();
        ffi::rlPopMatrix();
    }
}

fn draw_shape(rect: Rectangle, corners: &Corners, color: Color) {
    shape::fill(rect, corners, color, None);
}

fn draw_body(
    d: &mut RaylibDrawHandle,
    ctx: &DrawContext,
    rect: Rectangle,
    style: &ButtonStyle,
    look: &Look,
) {
    if let Some(shadow) = look.shadow {
        let offset = Rectangle::new(
            rect.x + shadow.offset.x,
            rect.y + shadow.offset.y,
            rect.width,
            rect.height,
        );
        draw_shape(offset, &style.corners, faded(shadow.color, look.opacity));
    }

    match &look.image {
        Some(image) => match ctx.resources.texture(&image.path) {
            Some(texture) => draw_image(d, texture, image, rect, faded(image.tint, look.opacity)),
            None => draw_shape(rect, &style.corners, faded(look.fill, look.opacity)),
        },
        None => draw_shape(rect, &style.corners, faded(look.fill, look.opacity)),
    }

    if let Some(border) = look.border
        && border.width > 0.0
    {
        shape::stroke(
            rect,
            &style.corners,
            border.width,
            faded(border.color, look.opacity),
        );
    }
}

fn draw_image(
    d: &mut RaylibDrawHandle,
    texture: &Texture2D,
    image: &ButtonImage,
    rect: Rectangle,
    tint: Color,
) {
    let source = Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32);
    match image.slice {
        Some(slice) => {
            let info = ffi::NPatchInfo {
                source: source.into(),
                left: slice.left,
                top: slice.top,
                right: slice.right,
                bottom: slice.bottom,
                layout: ffi::NPatchLayout::NPATCH_NINE_PATCH as i32,
            };
            d.draw_texture_n_patch(texture, info, rect, Vector2::zero(), 0.0, tint);
        }
        None => d.draw_texture_pro(texture, source, rect, Vector2::zero(), 0.0, tint),
    }
}

fn draw_content(
    d: &mut RaylibDrawHandle,
    ctx: &DrawContext,
    rect: Rectangle,
    label: &str,
    style: &ButtonStyle,
    look: &Look,
) {
    let fonts = ctx.fonts();
    let inner = Rectangle::new(
        rect.x + style.padding.x,
        rect.y + style.padding.y,
        (rect.width - style.padding.x * 2.0).max(0.0),
        (rect.height - style.padding.y * 2.0).max(0.0),
    );

    let text_color = faded(look.text_color, look.opacity);
    let icon = style.icon.as_ref().and_then(|icon| {
        let texture = ctx.resources.texture(&icon.path)?;
        let aspect = texture.width as f32 / texture.height.max(1) as f32;
        Some((icon, texture, icon.size * aspect))
    });
    let icon_space = match (&icon, label.is_empty()) {
        (Some((icon, _, width)), false) => width + icon.gap,
        (Some((_, _, width)), true) => *width,
        (None, _) => 0.0,
    };

    let mut text =
        TextStyle::new(style.text.font, style.text.size, text_color).spacing(style.text.spacing);
    let available = (inner.width - icon_space).max(0.0);
    let lines: Vec<String> = match style.overflow {
        _ if label.is_empty() => Vec::new(),
        TextOverflow::Overflow => vec![label.to_string()],
        TextOverflow::Ellipsis => vec![fit_text(fonts, &text, label, available)],
        TextOverflow::Shrink => {
            let width = crate::ui::measure_text(fonts, label, &text).x;
            if width > available && width > 0.0 {
                text.size = (text.size * available / width).max(8.0);
            }
            vec![label.to_string()]
        }
        TextOverflow::Wrap => fonts.wrap(text.font, label, text.size, available),
    };

    let widths: Vec<f32> = lines
        .iter()
        .map(|line| crate::ui::measure_text(fonts, line, &text).x)
        .collect();
    let text_width = widths.iter().copied().fold(0.0, f32::max);
    let group = text_width + icon_space;
    let start = match style.align {
        TextAlign::Left => inner.x,
        TextAlign::Center => inner.x + (inner.width - group) / 2.0,
        TextAlign::Right => inner.x + inner.width - group,
    };

    let icon_left = matches!(icon, Some((icon, _, _)) if icon.side == IconSide::Left);
    let text_x = if icon_left { start + icon_space } else { start };

    if let Some((icon, texture, width)) = icon {
        let x = if icon.side == IconSide::Left || label.is_empty() {
            start
        } else {
            start + text_width + icon.gap
        };
        let dest = Rectangle::new(
            x,
            rect.y + (rect.height - icon.size) / 2.0,
            width,
            icon.size,
        );
        let source = Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32);
        let tint = faded(icon.tint.unwrap_or(look.text_color), look.opacity);
        d.draw_texture_pro(texture, source, dest, Vector2::zero(), 0.0, tint);
    }

    let line_height = text.size * 1.25;
    let block = lines.len() as f32 * line_height - (line_height - text.size);
    let top = rect.y + (rect.height - block) / 2.0;
    for (i, (line, width)) in lines.iter().zip(&widths).enumerate() {
        let x = match style.align {
            TextAlign::Left => text_x,
            TextAlign::Center => text_x + (text_width - width) / 2.0,
            TextAlign::Right => text_x + text_width - width,
        };
        let y = top + i as f32 * line_height;
        crate::ui::draw_text(d, fonts, line, Vector2::new(x, y), &text);
        if let Some(line) = look.underline
            && line.width > 0.0
            && line.color.a > 0
        {
            let under = Rectangle::new(x, y + text.size + 3.0, *width, line.width);
            crate::shape::fill(
                under,
                &crate::Corners::SQUARE,
                faded(line.color, look.opacity),
                None,
            );
        }
    }
}
