use raylib::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct ScrollStyle {
    pub step: f32,
    pub bar_width: f32,
    pub bar_inset: f32,
    pub bar_color: Color,
    pub track_color: Color,
    pub min_thumb: f32,
}

impl Default for ScrollStyle {
    fn default() -> Self {
        Self {
            step: 48.0,
            bar_width: 6.0,
            bar_inset: 4.0,
            bar_color: Color::new(255, 255, 255, 70),
            track_color: Color::new(255, 255, 255, 20),
            min_thumb: 24.0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Scroll {
    offset: f32,
    view: f32,
    content: f32,
    dragging: bool,
    grab: f32,
}

impl Scroll {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extent(&mut self, view: f32, content: f32) {
        self.view = view.max(0.0);
        self.content = content.max(0.0);
        self.offset = self.offset.clamp(0.0, self.max());
    }

    pub fn max(&self) -> f32 {
        (self.content - self.view).max(0.0)
    }

    pub fn offset(&self) -> f32 {
        self.offset.min(self.max())
    }

    pub fn from_end(&self) -> f32 {
        self.max() - self.offset()
    }

    pub fn overflows(&self) -> bool {
        self.max() > 0.0
    }

    pub fn at_end(&self) -> bool {
        self.offset() >= self.max()
    }

    pub fn by(&mut self, delta: f32) {
        self.offset = (self.offset + delta).clamp(0.0, self.max());
    }

    pub fn page(&self) -> f32 {
        self.view * 0.9
    }

    pub fn to_start(&mut self) {
        self.offset = 0.0;
    }

    pub fn to_end(&mut self) {
        self.offset = self.max();
    }

    pub fn dragging(&self) -> bool {
        self.dragging
    }

    pub fn bar(&self, area: Rectangle, style: &ScrollStyle) -> Option<Rectangle> {
        if !self.overflows() {
            return None;
        }
        Some(Rectangle::new(
            area.x + area.width - style.bar_width - style.bar_inset,
            area.y,
            style.bar_width,
            area.height,
        ))
    }

    pub fn thumb(&self, area: Rectangle, style: &ScrollStyle) -> Option<Rectangle> {
        let bar = self.bar(area, style)?;
        if self.content <= 0.0 {
            return None;
        }
        let height = (bar.height * (self.view / self.content))
            .max(style.min_thumb)
            .min(bar.height);
        let travel = bar.height - height;
        let position = if self.max() > 0.0 {
            self.offset() / self.max()
        } else {
            0.0
        };
        Some(Rectangle::new(
            bar.x,
            bar.y + travel * position,
            bar.width,
            height,
        ))
    }

    pub fn input(&mut self, rl: &RaylibHandle, area: Rectangle, style: &ScrollStyle) {
        let mouse = crate::frame::viewport::mouse_position(rl);

        if let Some(thumb) = self.thumb(area, style) {
            let bar = self.bar(area, style).unwrap_or(thumb);
            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                if thumb.check_collision_point_rec(mouse) {
                    self.dragging = true;
                    self.grab = mouse.y - thumb.y;
                } else if bar.check_collision_point_rec(mouse) {
                    self.dragging = true;
                    self.grab = thumb.height / 2.0;
                }
            }
            if self.dragging {
                if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    let travel = bar.height - thumb.height;
                    let position = if travel > 0.0 {
                        ((mouse.y - self.grab - bar.y) / travel).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    self.offset = position * self.max();
                } else {
                    self.dragging = false;
                }
            }
        } else {
            self.dragging = false;
        }

        if area.check_collision_point_rec(mouse) {
            self.by(-rl.get_mouse_wheel_move() * style.step);
        }
    }

    pub fn draw_bar(&self, d: &mut RaylibDrawHandle, area: Rectangle, style: &ScrollStyle) {
        let (Some(bar), Some(thumb)) = (self.bar(area, style), self.thumb(area, style)) else {
            return;
        };
        let radius = style.bar_width / 2.0;
        d.draw_rectangle_rounded(bar, 1.0, 6, style.track_color);
        d.draw_rectangle_rounded(thumb, radius / thumb.height.max(1.0), 6, style.bar_color);
    }
}
