use raylib::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum CursorKind {
    #[default]
    Arrow,
    Hand,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CursorStyle {
    pub arrow: String,
    pub hand: Option<String>,
    pub size: f32,
    pub hotspot: Vector2,
}

impl CursorStyle {
    pub fn new(arrow: impl Into<String>) -> Self {
        Self {
            arrow: arrow.into(),
            hand: None,
            size: 32.0,
            hotspot: Vector2::zero(),
        }
    }

    pub fn hand(mut self, file: impl Into<String>) -> Self {
        self.hand = Some(file.into());
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        self
    }

    pub fn hotspot(mut self, x: f32, y: f32) -> Self {
        self.hotspot = Vector2::new(x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
        self
    }

    pub fn file(&self, kind: CursorKind) -> &str {
        match kind {
            CursorKind::Hand => self.hand.as_deref().unwrap_or(&self.arrow),
            CursorKind::Arrow => &self.arrow,
        }
    }

    pub fn rect(&self, at: Vector2, natural: Vector2) -> Rectangle {
        let scale = match natural.y > 0.0 {
            true => self.size / natural.y,
            false => 1.0,
        };
        let size = Vector2::new(natural.x * scale, natural.y * scale);
        Rectangle::new(
            at.x - size.x * self.hotspot.x,
            at.y - size.y * self.hotspot.y,
            size.x,
            size.y,
        )
    }
}

pub fn path(file: &str) -> String {
    match file.contains('/') {
        true => file.to_lowercase(),
        false => format!("ui/{}", file).to_lowercase(),
    }
}
