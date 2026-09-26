use std::cell::Cell;

use raylib::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub size: (i32, i32),
    pub destination: Rectangle,
}

impl Viewport {
    pub fn scale(&self) -> f32 {
        if self.size.0 <= 0 || self.destination.width <= 0.0 {
            return 1.0;
        }
        self.destination.width / self.size.0 as f32
    }

    pub fn to_viewport(&self, point: Vector2) -> Vector2 {
        let scale = self.scale();
        Vector2::new(
            (point.x - self.destination.x) / scale,
            (point.y - self.destination.y) / scale,
        )
    }
}

pub const UI_SCALES: [u32; 4] = [100, 110, 125, 150];

pub fn nearest_ui_scale(percent: u32) -> usize {
    UI_SCALES
        .iter()
        .enumerate()
        .min_by_key(|(_, scale)| scale.abs_diff(percent))
        .map_or(0, |(index, _)| index)
}

pub fn ui_factor(percent: u32) -> f32 {
    UI_SCALES[nearest_ui_scale(percent)] as f32 / 100.0
}

pub fn scaled_layout(layout: (i32, i32), percent: u32) -> (i32, i32) {
    let factor = ui_factor(percent);
    (
        (layout.0 as f32 / factor).round() as i32,
        (layout.1 as f32 / factor).round() as i32,
    )
}

thread_local! {
    static CURRENT: Cell<Option<Viewport>> = const { Cell::new(None) };
    static RENDER_SCALE: Cell<f32> = const { Cell::new(1.0) };
}

pub fn set(viewport: Viewport) {
    CURRENT.with(|current| current.set(Some(viewport)));
}

pub fn clear() {
    CURRENT.with(|current| current.set(None));
}

pub fn current() -> Option<Viewport> {
    CURRENT.with(|current| current.get())
}

pub fn size(rl: &RaylibHandle) -> Vector2 {
    match current() {
        Some(viewport) => Vector2::new(viewport.size.0 as f32, viewport.size.1 as f32),
        None => Vector2::new(rl.get_screen_width() as f32, rl.get_screen_height() as f32),
    }
}

pub fn render_scale() -> f32 {
    RENDER_SCALE.with(|scale| scale.get())
}

pub fn set_render_scale(scale: f32) {
    RENDER_SCALE.with(|current| current.set(scale.max(1.0)));
}

pub fn mouse_position(rl: &RaylibHandle) -> Vector2 {
    let point = rl.get_mouse_position();
    match current() {
        Some(viewport) => viewport.to_viewport(point),
        None => point,
    }
}
