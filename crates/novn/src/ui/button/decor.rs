use raylib::prelude::*;

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
