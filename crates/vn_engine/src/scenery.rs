use std::f64::consts::TAU;

use raylib::prelude::*;

use crate::shape::{self, Corners, Gradient, GradientDirection};
use crate::ui::{self, Background};
use crate::{Border, ResourceManager};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    pub zoom: f32,
    pub period: f64,
    pub pan: Vector2,
}

impl Motion {
    pub fn new(zoom: f32, period: f64) -> Self {
        Self {
            zoom: zoom.max(1.0),
            period: period.max(0.1),
            pan: Vector2::zero(),
        }
    }

    pub fn pan(mut self, x: f32, y: f32) -> Self {
        self.pan = Vector2::new(x.clamp(-1.0, 1.0), y.clamp(-1.0, 1.0));
        self
    }

    pub fn at(&self, time: f64) -> (f32, Vector2) {
        let phase = ((1.0 - (TAU * time / self.period).cos()) / 2.0) as f32;
        let zoom = 1.0 + (self.zoom - 1.0) * phase;
        let drift = phase * 2.0 - 1.0;
        (zoom, self.pan * drift)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Letterbox {
    pub height: f32,
    pub color: Color,
    pub rule: Option<Border>,
    pub slide_in: f64,
}

impl Default for Letterbox {
    fn default() -> Self {
        Self {
            height: 80.0,
            color: Color::BLACK,
            rule: None,
            slide_in: 0.0,
        }
    }
}

impl Letterbox {
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(0.0);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn rule(mut self, width: f32, color: Color) -> Self {
        self.rule = Some(Border::new(width, color));
        self
    }

    pub fn slide_in(mut self, seconds: f64) -> Self {
        self.slide_in = seconds.max(0.0);
        self
    }

    pub fn height_at(&self, since_open: f64) -> f32 {
        if self.slide_in <= 0.0 {
            return self.height;
        }
        let t = (since_open / self.slide_in).clamp(0.0, 1.0) as f32;
        self.height * (1.0 - (1.0 - t).powi(3))
    }

    pub fn bars(&self, screen: Vector2, since_open: f64) -> (Rectangle, Rectangle) {
        let height = self.height_at(since_open);
        (
            Rectangle::new(0.0, 0.0, screen.x, height),
            Rectangle::new(0.0, screen.y - height, screen.x, height),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vignette {
    pub color: Color,
    pub size: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Scenery {
    pub motion: Option<Motion>,
    pub letterbox: Option<Letterbox>,
    pub vignette: Option<Vignette>,
}

impl Scenery {
    pub fn motion(mut self, zoom: f32, period: f64) -> Self {
        let pan = self.motion.map_or(Vector2::zero(), |m| m.pan);
        self.motion = Some(Motion {
            pan,
            ..Motion::new(zoom, period)
        });
        self
    }

    pub fn pan(mut self, x: f32, y: f32) -> Self {
        self.motion = Some(self.motion.unwrap_or(Motion::new(1.0, 30.0)).pan(x, y));
        self
    }

    pub fn letterbox(mut self, letterbox: impl FnOnce(Letterbox) -> Letterbox) -> Self {
        self.letterbox = Some(letterbox(self.letterbox.unwrap_or_default()));
        self
    }

    pub fn vignette(mut self, color: Color, size: f32) -> Self {
        self.vignette = Some(Vignette {
            color,
            size: size.clamp(0.0, 0.5),
        });
        self
    }

    pub fn draw_background(
        &self,
        d: &mut RaylibDrawHandle,
        resources: &ResourceManager,
        background: Option<&Background>,
    ) {
        let (Some(motion), Some(Background::Image(path))) = (self.motion, background) else {
            ui::draw_background(d, resources, background);
            return;
        };
        let Some(texture) = resources.textures.get(path) else {
            return;
        };
        let screen = ui::screen_size(d);
        let (zoom, pan) = motion.at(d.get_time());
        let (w, h) = (texture.width as f32, texture.height as f32);
        let scale = (screen.x / w).max(screen.y / h) * zoom;
        let (source_w, source_h) = (screen.x / scale, screen.y / scale);
        let (slack_x, slack_y) = ((w - source_w) / 2.0, (h - source_h) / 2.0);
        let source = Rectangle::new(
            slack_x + slack_x * pan.x,
            slack_y + slack_y * pan.y,
            source_w,
            source_h,
        );
        let area = Rectangle::new(0.0, 0.0, screen.x, screen.y);
        d.draw_texture_pro(texture, source, area, Vector2::zero(), 0.0, Color::WHITE);
    }

    pub fn draw_frame(&self, d: &mut RaylibDrawHandle, since_open: f64) {
        let screen = ui::screen_size(d);
        if let Some(vignette) = self.vignette {
            let clear = Color::new(vignette.color.r, vignette.color.g, vignette.color.b, 0);
            let (w, h) = (screen.x * vignette.size, screen.y * vignette.size);
            let edges = [
                (
                    Rectangle::new(0.0, 0.0, screen.x, h),
                    vignette.color,
                    clear,
                    GradientDirection::Vertical,
                ),
                (
                    Rectangle::new(0.0, screen.y - h, screen.x, h),
                    clear,
                    vignette.color,
                    GradientDirection::Vertical,
                ),
                (
                    Rectangle::new(0.0, 0.0, w, screen.y),
                    vignette.color,
                    clear,
                    GradientDirection::Horizontal,
                ),
                (
                    Rectangle::new(screen.x - w, 0.0, w, screen.y),
                    clear,
                    vignette.color,
                    GradientDirection::Horizontal,
                ),
            ];
            for (rect, from, to, direction) in edges {
                shape::fill(
                    rect,
                    &Corners::SQUARE,
                    from,
                    Some(Gradient { to, direction }),
                );
            }
        }

        if let Some(letterbox) = self.letterbox {
            let (top, bottom) = letterbox.bars(screen, since_open);
            for bar in [top, bottom] {
                shape::fill(bar, &Corners::SQUARE, letterbox.color, None);
            }
            if let Some(rule) = letterbox.rule
                && top.height > 0.0
            {
                let lines = [
                    Rectangle::new(0.0, top.y + top.height - rule.width, screen.x, rule.width),
                    Rectangle::new(0.0, bottom.y, screen.x, rule.width),
                ];
                for line in lines {
                    shape::fill(line, &Corners::SQUARE, rule.color, None);
                }
            }
        }
    }

    pub fn bottom_bar(&self, screen: Vector2, since_open: f64) -> Option<Rectangle> {
        self.letterbox.map(|l| l.bars(screen, since_open).1)
    }
}
