use raylib::prelude::*;

pub struct Switch {
    pub rect: Rectangle,
    pub is_on: bool,
    pub label: String,
}

impl Switch {
    pub fn new(x: f32, y: f32, text: &str, initial: bool) -> Self {
        Self {
            rect: rrect(x, y, 50, 50),
            label: text.to_string(),
            is_on: initial,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle) {
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            && self.rect.check_collision_point_rec(rl.get_mouse_position())
        {
            self.is_on = !self.is_on;
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_text(
            &self.label,
            (self.rect.x - 100.0) as i32,
            self.rect.y as i32,
            20,
            Color::WHITE,
        );

        let track_color = if self.is_on {
            Color::LIME
        } else {
            Color::DARKGRAY
        };
        d.draw_rectangle_rounded(self.rect, 0.5, 10, track_color);

        let knob_x = if self.is_on {
            self.rect.x + 25.0
        } else {
            self.rect.x
        };
        d.draw_rectangle_rounded(rrect(knob_x, self.rect.y, 25, 25), 0.5, 10, Color::RAYWHITE);
    }
}
