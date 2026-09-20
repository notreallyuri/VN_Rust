use raylib::prelude::*;

use crate::shape;
use crate::ui::{self, TextStyle};
use crate::{Border, DrawContext, FontRole, PanelStyle};

#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Rect(Rectangle),
    Circle { center: Vector2, radius: f32 },
    Polygon(Vec<Vector2>),
}

impl Shape {
    pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Self {
        Shape::Rect(Rectangle::new(x, y, width, height))
    }

    pub fn all() -> Self {
        Shape::rect(0.0, 0.0, 1.0, 1.0)
    }

    pub fn circle(x: f32, y: f32, radius: f32) -> Self {
        Shape::Circle {
            center: Vector2::new(x, y),
            radius,
        }
    }

    pub fn polygon(points: impl IntoIterator<Item = (f32, f32)>) -> Self {
        Shape::Polygon(
            points
                .into_iter()
                .map(|(x, y)| Vector2::new(x, y))
                .collect(),
        )
    }

    pub fn in_image(self, width: f32, height: f32) -> Self {
        let (width, height) = (width.max(1.0), height.max(1.0));
        let scale = |point: Vector2| Vector2::new(point.x / width, point.y / height);
        match self {
            Shape::Rect(rect) => Shape::Rect(Rectangle::new(
                rect.x / width,
                rect.y / height,
                rect.width / width,
                rect.height / height,
            )),
            Shape::Circle { center, radius } => Shape::Circle {
                center: scale(center),
                radius: radius / width,
            },
            Shape::Polygon(points) => Shape::Polygon(points.into_iter().map(scale).collect()),
        }
    }

    pub fn bounds(&self, area: Rectangle) -> Rectangle {
        match self {
            Shape::Rect(rect) => Rectangle::new(
                area.x + rect.x * area.width,
                area.y + rect.y * area.height,
                rect.width * area.width,
                rect.height * area.height,
            ),
            Shape::Circle { center, radius } => {
                let center = at(area, *center);
                let radius = radius * area.width.abs();
                Rectangle::new(
                    center.x - radius,
                    center.y - radius,
                    radius * 2.0,
                    radius * 2.0,
                )
            }
            Shape::Polygon(points) => {
                let Some(first) = points.first().map(|point| at(area, *point)) else {
                    return Rectangle::new(area.x, area.y, 0.0, 0.0);
                };
                let (mut low, mut high) = (first, first);
                for point in points.iter().map(|point| at(area, *point)) {
                    low = Vector2::new(low.x.min(point.x), low.y.min(point.y));
                    high = Vector2::new(high.x.max(point.x), high.y.max(point.y));
                }
                Rectangle::new(low.x, low.y, high.x - low.x, high.y - low.y)
            }
        }
    }

    pub fn center(&self, area: Rectangle) -> Vector2 {
        match self {
            Shape::Circle { center, .. } => at(area, *center),
            _ => {
                let bounds = self.bounds(area);
                Vector2::new(
                    bounds.x + bounds.width / 2.0,
                    bounds.y + bounds.height / 2.0,
                )
            }
        }
    }

    pub fn contains(&self, area: Rectangle, point: Vector2) -> bool {
        match self {
            Shape::Rect(_) => self.bounds(area).check_collision_point_rec(point),
            Shape::Circle { center, radius } => {
                let center = at(area, *center);
                let radius = radius * area.width.abs();
                let (dx, dy) = (point.x - center.x, point.y - center.y);
                dx * dx + dy * dy <= radius * radius
            }
            Shape::Polygon(points) => {
                let points: Vec<Vector2> = points.iter().map(|point| at(area, *point)).collect();
                inside(&points, point)
            }
        }
    }

    pub fn draw_fill(&self, _d: &mut RaylibDrawHandle, area: Rectangle, color: Color) {
        if color.a == 0 {
            return;
        }
        match self {
            Shape::Rect(_) => {
                shape::fill(self.bounds(area), &crate::Corners::SQUARE, color, None);
            }
            Shape::Circle { .. } => {
                let bounds = self.bounds(area);
                let corners = crate::Corners::round(bounds.width / 2.0);
                shape::fill(bounds, &corners, color, None);
            }
            Shape::Polygon(points) => {
                let points: Vec<Vector2> = points.iter().map(|point| at(area, *point)).collect();
                let vertices: Vec<(Vector2, Color)> = triangulate(&points)
                    .into_iter()
                    .flatten()
                    .map(|point| (point, color))
                    .collect();
                shape::triangles(&vertices);
            }
        }
    }

    pub fn draw_border(
        &self,
        _d: &mut RaylibDrawHandle,
        area: Rectangle,
        width: f32,
        color: Color,
    ) {
        if color.a == 0 || width <= 0.0 {
            return;
        }
        match self {
            Shape::Rect(_) => {
                shape::stroke(self.bounds(area), &crate::Corners::SQUARE, width, color);
            }
            Shape::Circle { .. } => {
                let bounds = self.bounds(area);
                let corners = crate::Corners::round(bounds.width / 2.0);
                shape::stroke(bounds, &corners, width, color);
            }
            Shape::Polygon(points) => {
                let points: Vec<Vector2> = points.iter().map(|point| at(area, *point)).collect();
                let mut vertices = Vec::with_capacity(points.len() * 6);
                for i in 0..points.len() {
                    let (a, b) = (points[i], points[(i + 1) % points.len()]);
                    let edge = Vector2::new(b.x - a.x, b.y - a.y);
                    let length = (edge.x * edge.x + edge.y * edge.y).sqrt().max(0.001);
                    let out = Vector2::new(
                        -edge.y / length * width / 2.0,
                        edge.x / length * width / 2.0,
                    );
                    let inner = |p: Vector2| Vector2::new(p.x - out.x, p.y - out.y);
                    let outer = |p: Vector2| Vector2::new(p.x + out.x, p.y + out.y);
                    vertices.extend(
                        [inner(a), inner(b), outer(b), inner(a), outer(b), outer(a)]
                            .map(|point| (point, color)),
                    );
                }
                shape::triangles(&vertices);
            }
        }
    }
}

fn at(area: Rectangle, point: Vector2) -> Vector2 {
    Vector2::new(
        area.x + point.x * area.width,
        area.y + point.y * area.height,
    )
}

fn inside(points: &[Vector2], point: Vector2) -> bool {
    if points.len() < 3 {
        return false;
    }
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

pub fn signed_area(points: &[Vector2]) -> f32 {
    let mut sum = 0.0;
    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);
        sum += a.x * b.y - b.x * a.y;
    }
    sum / 2.0
}

pub fn triangulate(points: &[Vector2]) -> Vec<[Vector2; 3]> {
    if points.len() < 3 {
        return Vec::new();
    }

    let mut left: Vec<Vector2> = points.to_vec();
    if signed_area(&left) < 0.0 {
        left.reverse();
    }

    let mut triangles = Vec::with_capacity(left.len() - 2);
    let mut attempts = left.len() * left.len();
    while left.len() > 3 && attempts > 0 {
        attempts -= 1;
        let count = left.len();
        let ear = (0..count).find(|&i| {
            let corner = [
                left[(i + count - 1) % count],
                left[i],
                left[(i + 1) % count],
            ];
            convex(corner)
                && !left.iter().enumerate().any(|(j, &point)| {
                    j != i
                        && j != (i + count - 1) % count
                        && j != (i + 1) % count
                        && in_triangle(corner, point)
                })
        });
        let Some(i) = ear else { break };
        let count = left.len();
        triangles.push([
            left[(i + count - 1) % count],
            left[i],
            left[(i + 1) % count],
        ]);
        left.remove(i);
    }
    if left.len() == 3 {
        triangles.push([left[0], left[1], left[2]]);
    }
    triangles
}

fn cross(a: Vector2, b: Vector2, c: Vector2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn convex(corner: [Vector2; 3]) -> bool {
    cross(corner[0], corner[1], corner[2]) > 0.0
}

fn in_triangle(corner: [Vector2; 3], point: Vector2) -> bool {
    let a = cross(corner[0], corner[1], point);
    let b = cross(corner[1], corner[2], point);
    let c = cross(corner[2], corner[0], point);
    (a >= 0.0 && b >= 0.0 && c >= 0.0) || (a <= 0.0 && b <= 0.0 && c <= 0.0)
}

pub fn pick<'a>(
    shapes: impl IntoIterator<Item = &'a Shape>,
    area: Rectangle,
    point: Vector2,
) -> Option<usize> {
    shapes
        .into_iter()
        .enumerate()
        .filter(|(_, shape)| shape.contains(area, point))
        .map(|(index, _)| index)
        .last()
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Highlight {
    pub fill: Option<Color>,
    pub border: Option<Border>,
    pub image: Option<String>,
    pub tint: Option<Color>,
    pub opacity: Option<f32>,
}

impl Highlight {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.border = Some(Border::new(width, color));
        self
    }

    pub fn image(mut self, path: impl Into<String>) -> Self {
        self.image = Some(path.into());
        self
    }

    pub fn tint(mut self, color: Color) -> Self {
        self.tint = Some(color);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity.clamp(0.0, 1.0));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.fill.is_none() && self.border.is_none() && self.image.is_none()
    }

    pub fn over(&self, base: &Highlight) -> Highlight {
        Highlight {
            fill: self.fill.or(base.fill),
            border: self.border.or(base.border),
            image: self.image.clone().or_else(|| base.image.clone()),
            tint: self.tint.or(base.tint),
            opacity: match (self.opacity, base.opacity) {
                (Some(a), Some(b)) => Some(a * b),
                (a, b) => a.or(b),
            },
        }
    }

    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        ctx: &DrawContext,
        shape: &Shape,
        area: Rectangle,
    ) {
        let opacity = self.opacity.unwrap_or(1.0);
        if opacity <= 0.0 {
            return;
        }

        if let Some(color) = self.fill {
            shape.draw_fill(d, area, fade(color, opacity));
        }
        if let Some(path) = &self.image
            && let Some(texture) = ctx.resources.texture(path)
        {
            let bounds = shape.bounds(area);
            let source = Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32);
            let tint = fade(self.tint.unwrap_or(Color::WHITE), opacity);
            d.draw_texture_pro(texture, source, bounds, Vector2::zero(), 0.0, tint);
        }
        if let Some(border) = self.border {
            shape.draw_border(d, area, border.width, fade(border.color, opacity));
        }
    }
}

fn fade(color: Color, opacity: f32) -> Color {
    color.alpha(color.a as f32 / 255.0 * opacity.clamp(0.0, 1.0))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelAt {
    Hidden,
    Above,
    Below,
    Center,
    Pointer,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LabelStyle {
    pub at: LabelAt,
    pub text: TextStyle,
    pub panel: Option<PanelStyle>,
    pub padding: Vector2,
    pub gap: f32,
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self {
            at: LabelAt::Above,
            text: TextStyle::new(FontRole::Menu, 18.0, Color::RAYWHITE),
            panel: Some(PanelStyle::new(Color::new(14, 14, 22, 220)).roundness(0.2)),
            padding: Vector2::new(12.0, 6.0),
            gap: 10.0,
        }
    }
}

impl LabelStyle {
    pub fn at(mut self, at: LabelAt) -> Self {
        self.at = at;
        self
    }

    pub fn text(mut self, text: TextStyle) -> Self {
        self.text = text;
        self
    }

    pub fn panel(mut self, panel: Option<PanelStyle>) -> Self {
        self.panel = panel;
        self
    }

    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.padding = Vector2::new(x, y);
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn rect(&self, size: Vector2, bounds: Rectangle, pointer: Vector2) -> Rectangle {
        let (width, height) = (size.x + self.padding.x * 2.0, size.y + self.padding.y * 2.0);
        let centered = bounds.x + (bounds.width - width) / 2.0;
        match self.at {
            LabelAt::Above | LabelAt::Hidden => {
                Rectangle::new(centered, bounds.y - height - self.gap, width, height)
            }
            LabelAt::Below => {
                Rectangle::new(centered, bounds.y + bounds.height + self.gap, width, height)
            }
            LabelAt::Center => Rectangle::new(
                centered,
                bounds.y + (bounds.height - height) / 2.0,
                width,
                height,
            ),
            LabelAt::Pointer => Rectangle::new(
                pointer.x + self.gap,
                pointer.y - height - self.gap,
                width,
                height,
            ),
        }
    }

    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        ctx: &DrawContext,
        bounds: Rectangle,
        pointer: Vector2,
        text: &str,
    ) {
        if self.at == LabelAt::Hidden || text.is_empty() {
            return;
        }

        let fonts = ctx.fonts();
        let size = ui::measure_text(fonts, text, &self.text);
        let rect = clamp_to_screen(self.rect(size, bounds, pointer), ui::screen_size(d));

        if let Some(panel) = &self.panel {
            panel.draw(d, rect);
        }
        ui::draw_text(
            d,
            fonts,
            text,
            Vector2::new(rect.x + self.padding.x, rect.y + self.padding.y),
            &self.text,
        );
    }
}

fn clamp_to_screen(rect: Rectangle, screen: Vector2) -> Rectangle {
    let x = rect.x.clamp(4.0, (screen.x - rect.width - 4.0).max(4.0));
    let y = rect.y.clamp(4.0, (screen.y - rect.height - 4.0).max(4.0));
    Rectangle::new(x, y, rect.width, rect.height)
}
