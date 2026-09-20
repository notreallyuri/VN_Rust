use std::f32::consts::FRAC_PI_2;

use raylib::ffi;
use raylib::prelude::*;

use crate::button::{Border, Shadow};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CornerShape {
    #[default]
    Square,
    Round,
    Bevel,
    Scoop,
    Notch,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Corner {
    pub shape: CornerShape,
    pub size: f32,
}

impl Corner {
    pub const SQUARE: Corner = Corner {
        shape: CornerShape::Square,
        size: 0.0,
    };

    pub fn new(shape: CornerShape, size: f32) -> Self {
        Self {
            shape,
            size: size.max(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Corners {
    pub top_left: Corner,
    pub top_right: Corner,
    pub bottom_right: Corner,
    pub bottom_left: Corner,
    pub relative: bool,
}

impl Corners {
    pub const SQUARE: Corners = Corners {
        top_left: Corner::SQUARE,
        top_right: Corner::SQUARE,
        bottom_right: Corner::SQUARE,
        bottom_left: Corner::SQUARE,
        relative: false,
    };

    pub fn all(shape: CornerShape, size: f32) -> Self {
        let corner = Corner::new(shape, size);
        Self {
            top_left: corner,
            top_right: corner,
            bottom_right: corner,
            bottom_left: corner,
            relative: false,
        }
    }

    pub fn square() -> Self {
        Self::SQUARE
    }

    pub fn round(radius: f32) -> Self {
        Self::all(CornerShape::Round, radius)
    }

    pub fn bevel(size: f32) -> Self {
        Self::all(CornerShape::Bevel, size)
    }

    pub fn scoop(radius: f32) -> Self {
        Self::all(CornerShape::Scoop, radius)
    }

    pub fn notch(size: f32) -> Self {
        Self::all(CornerShape::Notch, size)
    }

    pub fn roundness(roundness: f32) -> Self {
        Self {
            relative: true,
            ..Self::all(CornerShape::Round, roundness.clamp(0.0, 1.0))
        }
    }

    pub fn top_left(mut self, shape: CornerShape, size: f32) -> Self {
        self.top_left = Corner::new(shape, size);
        self
    }

    pub fn top_right(mut self, shape: CornerShape, size: f32) -> Self {
        self.top_right = Corner::new(shape, size);
        self
    }

    pub fn bottom_right(mut self, shape: CornerShape, size: f32) -> Self {
        self.bottom_right = Corner::new(shape, size);
        self
    }

    pub fn bottom_left(mut self, shape: CornerShape, size: f32) -> Self {
        self.bottom_left = Corner::new(shape, size);
        self
    }

    pub fn top(self, shape: CornerShape, size: f32) -> Self {
        self.top_left(shape, size).top_right(shape, size)
    }

    pub fn bottom(self, shape: CornerShape, size: f32) -> Self {
        self.bottom_left(shape, size).bottom_right(shape, size)
    }

    pub fn is_square(&self) -> bool {
        self.list()
            .iter()
            .all(|c| c.shape == CornerShape::Square || c.size <= 0.0)
    }

    fn list(&self) -> [Corner; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
    }

    pub fn resolved(&self, rect: Rectangle) -> [Corner; 4] {
        let short = rect.width.min(rect.height).max(0.0);
        self.list().map(|corner| {
            let size = if self.relative {
                corner.size * short / 2.0
            } else {
                corner.size
            };
            Corner::new(corner.shape, size.min(short / 2.0))
        })
    }

    pub fn outline(&self, rect: Rectangle) -> Vec<Vector2> {
        outline(rect, &self.resolved(rect), 0.0, None)
    }
}

impl From<CornerShape> for Corners {
    fn from(shape: CornerShape) -> Self {
        Corners::all(shape, 8.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientDirection {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gradient {
    pub to: Color,
    pub direction: GradientDirection,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelStyle {
    pub color: Color,
    pub gradient: Option<Gradient>,
    pub corners: Corners,
    pub border: Option<Border>,
    pub inner_border: Option<(f32, Border)>,
    pub shadow: Option<Shadow>,
}

impl Default for PanelStyle {
    fn default() -> Self {
        Self::new(Color::new(20, 20, 30, 235))
    }
}

impl PanelStyle {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            gradient: None,
            corners: Corners::SQUARE,
            border: None,
            inner_border: None,
            shadow: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn gradient(mut self, to: Color, direction: GradientDirection) -> Self {
        self.gradient = Some(Gradient { to, direction });
        self
    }

    pub fn corners(mut self, corners: impl Into<Corners>) -> Self {
        self.corners = corners.into();
        self
    }

    pub fn roundness(self, roundness: f32) -> Self {
        self.corners(Corners::roundness(roundness))
    }

    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.border = Some(Border::new(width, color));
        self
    }

    pub fn no_border(mut self) -> Self {
        self.border = None;
        self
    }

    pub fn inner_border(mut self, inset: f32, width: f32, color: Color) -> Self {
        self.inner_border = Some((inset, Border::new(width, color)));
        self
    }

    pub fn shadow(mut self, x: f32, y: f32, color: Color) -> Self {
        self.shadow = Some(Shadow::new(x, y, color));
        self
    }

    pub fn draw(&self, d: &mut impl RaylibDraw, rect: Rectangle) {
        self.draw_faded(d, rect, 1.0);
    }

    pub fn draw_faded(&self, _d: &mut impl RaylibDraw, rect: Rectangle, opacity: f32) {
        if let Some(shadow) = self.shadow {
            let offset = Rectangle::new(
                rect.x + shadow.offset.x,
                rect.y + shadow.offset.y,
                rect.width,
                rect.height,
            );
            fill(offset, &self.corners, fade(shadow.color, opacity), None);
        }
        let gradient = self.gradient.map(|g| Gradient {
            to: fade(g.to, opacity),
            ..g
        });
        fill(rect, &self.corners, fade(self.color, opacity), gradient);
        if let Some(border) = self.border {
            stroke(
                rect,
                &self.corners,
                border.width,
                fade(border.color, opacity),
            );
        }
        if let Some((inset, border)) = self.inner_border {
            let inner = inset_rect(rect, inset);
            let corners = inset_corners(&self.corners, rect, inset);
            stroke(inner, &corners, border.width, fade(border.color, opacity));
        }
    }
}

fn fade(color: Color, opacity: f32) -> Color {
    Color::new(
        color.r,
        color.g,
        color.b,
        (color.a as f32 * opacity.clamp(0.0, 1.0)).round() as u8,
    )
}

fn inset_rect(rect: Rectangle, inset: f32) -> Rectangle {
    Rectangle::new(
        rect.x + inset,
        rect.y + inset,
        (rect.width - inset * 2.0).max(0.0),
        (rect.height - inset * 2.0).max(0.0),
    )
}

fn inset_corners(corners: &Corners, rect: Rectangle, inset: f32) -> Corners {
    let [top_left, top_right, bottom_right, bottom_left] = corners.resolved(rect).map(|c| {
        let size = match c.shape {
            CornerShape::Square => 0.0,
            CornerShape::Round | CornerShape::Bevel => c.size - inset,
            CornerShape::Scoop | CornerShape::Notch => c.size + inset,
        };
        Corner::new(c.shape, size)
    });
    Corners {
        top_left,
        top_right,
        bottom_right,
        bottom_left,
        relative: false,
    }
}

fn segments(corner: &Corner) -> usize {
    let scale = crate::viewport::render_scale().max(1.0);
    ((corner.size * scale / 1.2).ceil() as usize).clamp(8, 96)
}

fn corner_points(corner: &Corner, width: f32, steps: usize) -> Vec<(f32, f32)> {
    let s = corner.size;
    let w = width;
    if s <= 0.0 {
        return vec![(w, w)];
    }
    match corner.shape {
        CornerShape::Square => vec![(w, w)],
        CornerShape::Bevel => {
            let s = (s - w * (2.0 - std::f32::consts::SQRT_2)).max(0.0);
            vec![(w, w + s), (w + s, w)]
        }
        CornerShape::Notch => vec![(w, s + w), (s + w, s + w), (s + w, w)],
        CornerShape::Round => {
            let c = s.max(w);
            let r = (s - w).max(0.0);
            (0..=steps)
                .map(|i| {
                    let t = FRAC_PI_2 * i as f32 / steps as f32;
                    (c - r * t.cos(), c - r * t.sin())
                })
                .collect()
        }
        CornerShape::Scoop => {
            let r = s + w;
            let t0 = (w / r).clamp(0.0, 1.0).asin();
            (0..=steps)
                .map(|i| {
                    let t = t0 + (FRAC_PI_2 - 2.0 * t0) * i as f32 / steps as f32;
                    (r * t.sin(), r * t.cos())
                })
                .collect()
        }
    }
}

fn outline(
    rect: Rectangle,
    corners: &[Corner; 4],
    width: f32,
    steps: Option<&[usize; 4]>,
) -> Vec<Vector2> {
    corner_outlines(rect, corners, width, steps)
        .into_iter()
        .flatten()
        .collect()
}

fn corner_outlines(
    rect: Rectangle,
    corners: &[Corner; 4],
    width: f32,
    steps: Option<&[usize; 4]>,
) -> [Vec<Vector2>; 4] {
    let frames = [
        (
            Vector2::new(rect.x, rect.y),
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 0.0),
        ),
        (
            Vector2::new(rect.x + rect.width, rect.y),
            Vector2::new(-1.0, 0.0),
            Vector2::new(0.0, 1.0),
        ),
        (
            Vector2::new(rect.x + rect.width, rect.y + rect.height),
            Vector2::new(0.0, -1.0),
            Vector2::new(-1.0, 0.0),
        ),
        (
            Vector2::new(rect.x, rect.y + rect.height),
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, -1.0),
        ),
    ];

    std::array::from_fn(|i| {
        let (k, a, b) = frames[i];
        let steps = steps.map_or_else(|| segments(&corners[i]), |s| s[i]);
        corner_points(&corners[i], width, steps)
            .into_iter()
            .map(|(u, v)| k + b * u + a * v)
            .collect()
    })
}

const FEATHER: f32 = 1.0;

fn feather(
    rect: Rectangle,
    corners: &[Corner; 4],
    edge: f32,
    outward: f32,
    color: impl Fn(Vector2) -> Color,
    vertices: &mut Vec<(Vector2, Color)>,
) {
    let steps = corners.map(|c| segments(&c));
    let solid = corner_outlines(rect, corners, edge, Some(&steps));
    let faded = corner_outlines(rect, corners, edge + outward, Some(&steps));
    for (solid, faded) in solid.iter().zip(&faded) {
        for i in 1..solid.len() {
            let (a, b) = (solid[i - 1], solid[i]);
            let (fa, fb) = (faded[i - 1], faded[i]);
            let straight = (a.x - b.x).abs() < 0.01 || (a.y - b.y).abs() < 0.01;
            if straight {
                continue;
            }
            let clear = |p: Vector2| {
                let c = color(p);
                (p, Color::new(c.r, c.g, c.b, 0))
            };
            vertices.extend([
                (a, color(a)),
                clear(fb),
                (b, color(b)),
                (a, color(a)),
                clear(fa),
                clear(fb),
            ]);
        }
    }
}

fn color_at(point: Vector2, rect: Rectangle, color: Color, gradient: Option<Gradient>) -> Color {
    let Some(gradient) = gradient else {
        return color;
    };
    let t = match gradient.direction {
        GradientDirection::Vertical => (point.y - rect.y) / rect.height.max(1.0),
        GradientDirection::Horizontal => (point.x - rect.x) / rect.width.max(1.0),
    }
    .clamp(0.0, 1.0);
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    Color::new(
        mix(color.r, gradient.to.r),
        mix(color.g, gradient.to.g),
        mix(color.b, gradient.to.b),
        mix(color.a, gradient.to.a),
    )
}

fn triangles(vertices: &[(Vector2, Color)]) {
    unsafe {
        ffi::rlSetTexture(0);
        ffi::rlBegin(ffi::RL_TRIANGLES as i32);
        for &[a, mut b, mut c] in vertices.as_chunks::<3>().0 {
            let cross = (b.0.x - a.0.x) * (c.0.y - a.0.y) - (b.0.y - a.0.y) * (c.0.x - a.0.x);
            if cross > 0.0 {
                std::mem::swap(&mut b, &mut c);
            }
            for (point, color) in [a, b, c] {
                ffi::rlColor4ub(color.r, color.g, color.b, color.a);
                ffi::rlVertex2f(point.x, point.y);
            }
        }
        ffi::rlEnd();
    }
}

pub fn fill(rect: Rectangle, corners: &Corners, color: Color, gradient: Option<Gradient>) {
    let transparent = color.a == 0 && gradient.is_none_or(|g| g.to.a == 0);
    if transparent || rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    let points = outline(rect, &corners.resolved(rect), 0.0, None);
    let center = Vector2::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    let at = |p: Vector2| (p, color_at(p, rect, color, gradient));

    let mut vertices = Vec::with_capacity(points.len() * 3);
    for i in 0..points.len() {
        let next = points[(i + 1) % points.len()];
        vertices.extend([at(center), at(next), at(points[i])]);
    }
    let resolved = corners.resolved(rect);
    feather(
        rect,
        &resolved,
        0.0,
        -FEATHER,
        |p| color_at(p, rect, color, gradient),
        &mut vertices,
    );
    triangles(&vertices);
}

pub fn stroke(rect: Rectangle, corners: &Corners, width: f32, color: Color) {
    if color.a == 0 || width <= 0.0 || rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    let resolved = corners.resolved(rect);
    let steps = resolved.map(|c| segments(&c));
    let width = width.min(rect.width.min(rect.height) / 2.0);
    let outer = outline(rect, &resolved, 0.0, Some(&steps));
    let inner = outline(rect, &resolved, width, Some(&steps));

    let mut vertices = Vec::with_capacity(outer.len() * 6);
    for i in 0..outer.len() {
        let j = (i + 1) % outer.len();
        vertices.extend(
            [outer[i], inner[j], outer[j], outer[i], inner[i], inner[j]].map(|p| (p, color)),
        );
    }
    feather(rect, &resolved, 0.0, -FEATHER, |_| color, &mut vertices);
    feather(rect, &resolved, width, FEATHER, |_| color, &mut vertices);
    triangles(&vertices);
}

pub fn contains(rect: Rectangle, corners: &Corners, point: Vector2) -> bool {
    if !rect.check_collision_point_rec(point) {
        return false;
    }
    let points = corners.outline(rect);
    let mut inside = false;
    let mut j = points.len() - 1;
    for i in 0..points.len() {
        let (a, b) = (points[i], points[j]);
        if (a.y > point.y) != (b.y > point.y)
            && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}
