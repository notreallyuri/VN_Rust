use std::f64::consts::TAU;

use raylib::prelude::*;

use crate::data::resources::ResourceManager;
use crate::ui;
use crate::ui::Background;
use crate::ui::button::Border;
use crate::ui::shape;
use crate::ui::shape::{Corners, Gradient, GradientDirection};

const STILL_WEATHER: f64 = 30.0;
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
        if crate::ui::motion::reduced() {
            return (1.0 + (self.zoom - 1.0) / 2.0, Vector2::zero());
        }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeatherKind {
    Rain,
    Snow,
    Dust,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weather {
    pub kind: WeatherKind,
    pub count: u32,
    pub speed: f32,
    pub drift: f32,
    pub sway: f32,
    pub size: f32,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
    pub at: Vector2,
    pub size: f32,
    pub alpha: f32,
}

const MAX_PARTICLES: u32 = 4000;

impl Weather {
    pub fn rain(count: u32) -> Self {
        Self {
            kind: WeatherKind::Rain,
            count: count.min(MAX_PARTICLES),
            speed: 1.6,
            drift: 0.0,
            sway: 0.0,
            size: 18.0,
            color: Color::new(180, 200, 225, 150),
        }
    }

    pub fn snow(count: u32) -> Self {
        Self {
            kind: WeatherKind::Snow,
            count: count.min(MAX_PARTICLES),
            speed: 0.12,
            drift: 0.01,
            sway: 0.03,
            size: 3.5,
            color: Color::new(240, 245, 255, 210),
        }
    }

    pub fn dust(count: u32) -> Self {
        Self {
            kind: WeatherKind::Dust,
            count: count.min(MAX_PARTICLES),
            speed: 0.02,
            drift: 0.006,
            sway: 0.02,
            size: 2.0,
            color: Color::new(255, 240, 200, 90),
        }
    }

    pub fn count(mut self, count: u32) -> Self {
        self.count = count.min(MAX_PARTICLES);
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn drift(mut self, drift: f32) -> Self {
        self.drift = drift;
        self
    }

    pub fn sway(mut self, sway: f32) -> Self {
        self.sway = sway.max(0.0);
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(0.0);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn particle(&self, index: u32, time: f64, screen: Vector2) -> Particle {
        let seed = noise(index);
        let phase = noise(index ^ 0x9e37_79b9);
        let depth = noise(index ^ 0x85eb_ca6b);

        let speed = self.speed * (0.7 + 0.6 * depth);
        let y = wrap(phase + (time * speed as f64) as f32);
        let swing = self.sway * ((TAU * (time * 0.25 + phase as f64)).sin() as f32);
        let x = wrap(seed + (time * self.drift as f64) as f32 + swing);

        Particle {
            at: Vector2::new(x * screen.x, y * screen.y),
            size: self.size * (0.6 + 0.8 * depth),
            alpha: 0.45 + 0.55 * depth,
        }
    }

    pub fn particles(
        &self,
        time: f64,
        screen: Vector2,
    ) -> impl Iterator<Item = Particle> + use<'_> {
        (0..self.count).map(move |index| self.particle(index, time, screen))
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, time: f64, screen: Vector2) {
        if self.count == 0 || self.size <= 0.0 || screen.x <= 0.0 || screen.y <= 0.0 {
            return;
        }
        for particle in self.particles(time, screen) {
            let color = Color::new(
                self.color.r,
                self.color.g,
                self.color.b,
                (self.color.a as f32 * particle.alpha) as u8,
            );
            match self.kind {
                WeatherKind::Rain => {
                    let tail = Vector2::new(
                        particle.at.x - self.drift.signum() * particle.size * 0.35,
                        particle.at.y - particle.size,
                    );
                    d.draw_line_ex(tail, particle.at, (particle.size * 0.08).max(1.0), color);
                }
                WeatherKind::Snow | WeatherKind::Dust => {
                    d.draw_circle_v(particle.at, particle.size, color)
                }
            }
        }
    }
}

fn noise(index: u32) -> f32 {
    let mut hash = index.wrapping_mul(0x9e37_79b1) ^ 0x85eb_ca6b;
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x2545_f491);
    hash ^= hash >> 13;
    (hash % 1_000_003) as f32 / 1_000_003.0
}

fn wrap(value: f32) -> f32 {
    let fraction = value - value.floor();
    if fraction.is_finite() {
        fraction.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Scenery {
    pub motion: Option<Motion>,
    pub letterbox: Option<Letterbox>,
    pub vignette: Option<Vignette>,
    pub weather: Option<Weather>,
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

    pub fn weather(mut self, weather: Weather) -> Self {
        self.weather = Some(weather);
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
        if let Some(weather) = self.weather {
            let time = if crate::ui::motion::reduced() {
                STILL_WEATHER
            } else {
                since_open
            };
            weather.draw(d, time, screen);
        }
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
