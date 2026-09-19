use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use raylib::ffi;
use raylib::prelude::*;

use crate::ui::{TextStyle, fit_text, lighten};
use crate::{DrawContext, FontRole, GameContext};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Border {
    pub width: f32,
    pub color: Color,
}

impl Border {
    pub fn new(width: f32, color: Color) -> Self {
        Self { width, color }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub offset: Vector2,
    pub color: Color,
}

impl Shadow {
    pub fn new(x: f32, y: f32, color: Color) -> Self {
        Self {
            offset: Vector2::new(x, y),
            color,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Slice {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonImage {
    pub path: String,
    pub slice: Option<Slice>,
    pub tint: Color,
}

impl ButtonImage {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            slice: None,
            tint: Color::WHITE,
        }
    }

    pub fn nine_slice(mut self, left: i32, top: i32, right: i32, bottom: i32) -> Self {
        self.slice = Some(Slice {
            left,
            top,
            right,
            bottom,
        });
        self
    }

    pub fn slice_all(self, border: i32) -> Self {
        self.nine_slice(border, border, border, border)
    }

    pub fn tint(mut self, color: Color) -> Self {
        self.tint = color;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconSide {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonIcon {
    pub path: String,
    pub size: f32,
    pub side: IconSide,
    pub gap: f32,
    pub tint: Option<Color>,
}

impl ButtonIcon {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            size: 22.0,
            side: IconSide::Left,
            gap: 10.0,
            tint: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
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

    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextOverflow {
    Overflow,
    Ellipsis,
    Shrink,
    Wrap,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub scale: Vector2,
    pub rotation: f32,
    pub skew: Vector2,
    pub offset: Vector2,
    pub origin: Vector2,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        scale: Vector2 { x: 1.0, y: 1.0 },
        rotation: 0.0,
        skew: Vector2 { x: 0.0, y: 0.0 },
        offset: Vector2 { x: 0.0, y: 0.0 },
        origin: Vector2 { x: 0.5, y: 0.5 },
    };

    pub fn scale(mut self, factor: f32) -> Self {
        self.scale = Vector2::new(factor, factor);
        self
    }

    pub fn scale_xy(mut self, x: f32, y: f32) -> Self {
        self.scale = Vector2::new(x, y);
        self
    }

    pub fn rotate(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    pub fn skew(mut self, x_degrees: f32, y_degrees: f32) -> Self {
        self.skew = Vector2::new(x_degrees, y_degrees);
        self
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Vector2::new(x, y);
        self
    }

    pub fn origin(mut self, x: f32, y: f32) -> Self {
        self.origin = Vector2::new(x, y);
        self
    }

    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
            || (self.scale == Self::IDENTITY.scale
                && self.rotation == 0.0
                && self.skew == Self::IDENTITY.skew
                && self.offset == Self::IDENTITY.offset)
    }

    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        let v = |a: Vector2, b: Vector2| Vector2::new(mix(a.x, b.x, t), mix(a.y, b.y, t));
        Transform {
            scale: v(self.scale, other.scale),
            rotation: mix(self.rotation, other.rotation, t),
            skew: v(self.skew, other.skew),
            offset: v(self.offset, other.offset),
            origin: v(self.origin, other.origin),
        }
    }

    fn pivot(&self, rect: Rectangle) -> Vector2 {
        Vector2::new(
            rect.x + rect.width * self.origin.x,
            rect.y + rect.height * self.origin.y,
        )
    }

    fn shear(&self) -> (f32, f32) {
        (
            self.skew.x.to_radians().tan(),
            self.skew.y.to_radians().tan(),
        )
    }

    pub fn apply(&self, rect: Rectangle, point: Vector2) -> Vector2 {
        let pivot = self.pivot(rect);
        let (sx, sy) = self.shear();
        let local = Vector2::new(
            (point.x - pivot.x) * self.scale.x,
            (point.y - pivot.y) * self.scale.y,
        );
        let sheared = Vector2::new(local.x + sx * local.y, local.y + sy * local.x);
        let (sin, cos) = self.rotation.to_radians().sin_cos();
        Vector2::new(
            sheared.x * cos - sheared.y * sin + pivot.x + self.offset.x,
            sheared.x * sin + sheared.y * cos + pivot.y + self.offset.y,
        )
    }

    pub fn invert(&self, rect: Rectangle, point: Vector2) -> Option<Vector2> {
        let pivot = self.pivot(rect);
        let (sx, sy) = self.shear();
        let determinant = 1.0 - sx * sy;
        if determinant.abs() < 1e-6 || self.scale.x == 0.0 || self.scale.y == 0.0 {
            return None;
        }

        let moved = Vector2::new(
            point.x - pivot.x - self.offset.x,
            point.y - pivot.y - self.offset.y,
        );
        let (sin, cos) = self.rotation.to_radians().sin_cos();
        let unrotated = Vector2::new(
            moved.x * cos + moved.y * sin,
            -moved.x * sin + moved.y * cos,
        );
        let unsheared = Vector2::new(
            (unrotated.x - sx * unrotated.y) / determinant,
            (unrotated.y - sy * unrotated.x) / determinant,
        );
        Some(Vector2::new(
            unsheared.x / self.scale.x + pivot.x,
            unsheared.y / self.scale.y + pivot.y,
        ))
    }

    pub fn contains(&self, rect: Rectangle, point: Vector2) -> bool {
        self.invert(rect, point)
            .is_some_and(|local| rect.check_collision_point_rec(local))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ButtonLook {
    pub fill: Option<Color>,
    pub text_color: Option<Color>,
    pub border: Option<Border>,
    pub shadow: Option<Shadow>,
    pub image: Option<ButtonImage>,
    pub transform: Option<Transform>,
    pub opacity: Option<f32>,
}

impl ButtonLook {
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
    pub roundness: f32,
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
            roundness: 0.0,
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

    pub fn roundness(mut self, roundness: f32) -> Self {
        self.roundness = roundness.clamp(0.0, 1.0);
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
}

impl Look {
    fn blend(&mut self, other: &ButtonLook, t: f32) {
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
    }
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
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

fn faded(color: Color, opacity: f32) -> Color {
    Color::new(
        color.r,
        color.g,
        color.b,
        (color.a as f32 * opacity).round() as u8,
    )
}

#[derive(Clone, Copy, Debug)]
struct Anim {
    amounts: StateAmounts,
    seen: f64,
}

thread_local! {
    static ANIMATIONS: RefCell<(f64, HashMap<u64, Anim>)> = RefCell::new((0.0, HashMap::new()));
    static PRESSED: RefCell<Option<u64>> = const { RefCell::new(None) };
}

fn button_key(rect: Rectangle, label: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for value in [rect.x, rect.y, rect.width, rect.height] {
        value.to_bits().hash(&mut hasher);
    }
    label.hash(&mut hasher);
    hasher.finish()
}

fn rect_key(rect: Rectangle) -> u64 {
    button_key(rect, "")
}

pub fn step_amount(current: f32, target: f32, dt: f32, seconds: f32) -> f32 {
    if seconds <= 0.0 {
        return target;
    }
    let delta = dt / seconds;
    if current < target {
        (current + delta).min(target)
    } else {
        (current - delta).max(target)
    }
}

pub fn button_hovered(rl: &RaylibHandle, rect: Rectangle, style: &ButtonStyle) -> bool {
    style.contains(rect, rl.get_mouse_position())
}

pub fn button_clicked(ctx: &mut GameContext, rect: Rectangle, style: &ButtonStyle) -> bool {
    let rl = &*ctx.rl;
    let mouse = rl.get_mouse_position();
    let hovered = style.contains(rect, mouse);
    let key = rect_key(rect);

    let before = mouse - rl.get_mouse_delta();
    let entered = hovered && !style.contains(rect, before);

    let pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
    let released = rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT);

    if pressed && hovered {
        PRESSED.with(|p| *p.borrow_mut() = Some(key));
    }
    let clicked = released && hovered && PRESSED.with(|p| *p.borrow() == Some(key));
    if released && PRESSED.with(|p| *p.borrow() == Some(key)) {
        PRESSED.with(|p| *p.borrow_mut() = None);
    }

    if entered && let Some(id) = &style.hover_sound {
        ctx.play_sound(id);
    }
    if clicked && let Some(id) = &style.click_sound {
        ctx.play_sound(id);
    }
    clicked
}

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
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str, style: &'a ButtonStyle) -> Self {
        Self {
            label,
            style,
            disabled: false,
            focused: false,
        }
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
        let hovered =
            ctx.interactive && !self.disabled && style.contains(rect, d.get_mouse_position());
        let held = d.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
        let target = StateAmounts {
            hover: f32::from(u8::from(hovered)),
            press: f32::from(u8::from(hovered && held)),
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
        let look = self.style.look(self.amounts(d, ctx, rect));
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

fn draw_shape(d: &mut RaylibDrawHandle, rect: Rectangle, roundness: f32, color: Color) {
    if color.a == 0 {
        return;
    }
    if roundness > 0.0 {
        d.draw_rectangle_rounded(rect, roundness, 8, color);
    } else {
        d.draw_rectangle_rec(rect, color);
    }
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
        draw_shape(
            d,
            offset,
            style.roundness,
            faded(shadow.color, look.opacity),
        );
    }

    match &look.image {
        Some(image) => match ctx.resources.texture(&image.path) {
            Some(texture) => draw_image(d, texture, image, rect, faded(image.tint, look.opacity)),
            None => draw_shape(d, rect, style.roundness, faded(look.fill, look.opacity)),
        },
        None => draw_shape(d, rect, style.roundness, faded(look.fill, look.opacity)),
    }

    if let Some(border) = look.border
        && border.width > 0.0
    {
        let color = faded(border.color, look.opacity);
        if style.roundness > 0.0 {
            d.draw_rectangle_rounded_lines_ex(rect, style.roundness, 8, border.width, color);
        } else {
            d.draw_rectangle_lines_ex(rect, border.width, color);
        }
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

    let mut text = TextStyle::new(style.text.font, style.text.size, text_color);
    let available = (inner.width - icon_space).max(0.0);
    let lines: Vec<String> = match style.overflow {
        _ if label.is_empty() => Vec::new(),
        TextOverflow::Overflow => vec![label.to_string()],
        TextOverflow::Ellipsis => vec![fit_text(fonts, &text, label, available)],
        TextOverflow::Shrink => {
            let width = fonts.measure(text.font, label, text.size).x;
            if width > available && width > 0.0 {
                text.size = (text.size * available / width).max(8.0);
            }
            vec![label.to_string()]
        }
        TextOverflow::Wrap => fonts.wrap(text.font, label, text.size, available),
    };

    let widths: Vec<f32> = lines
        .iter()
        .map(|line| fonts.measure(text.font, line, text.size).x)
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
        fonts.draw(
            d,
            text.font,
            line,
            Vector2::new(x, y),
            text.size,
            text.color,
        );
    }
}
