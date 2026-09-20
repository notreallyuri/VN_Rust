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

thread_local! {
    static CURRENT: Cell<Option<Viewport>> = const { Cell::new(None) };
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

pub fn mouse_position(rl: &RaylibHandle) -> Vector2 {
    let point = rl.get_mouse_position();
    match current() {
        Some(viewport) => viewport.to_viewport(point),
        None => point,
    }
}
