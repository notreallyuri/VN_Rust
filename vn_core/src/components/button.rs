use raylib::prelude::*;

pub struct Button {
    pub rect: Rectangle,
    pub text: String,
    pub color: Color,
    pub hover_color: Color,
}

impl Button {
    pub fn new(x: f32, y: f32, w: f32, h: f32, text: &str, color: Color) -> Self {
        Self {
            rect: rrect(x, y, w, h),
            text: text.to_string(),
            color,
            hover_color: Color::alpha(&color, 0.85),
        }
    }

    pub fn is_clicked(&self, rl: &RaylibHandle) -> bool {
        let mouse = rl.get_mouse_position();

        self.rect.check_collision_point_rec(mouse)
            && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let mouse = d.get_mouse_position();

        let current_color = if self.rect.check_collision_point_rec(mouse) {
            self.hover_color
        } else {
            self.color
        };

        d.draw_rectangle_rec(self.rect, current_color);

        let font_size = 20;
        let text_width = d.measure_text(&self.text, font_size);
        let text_x = self.rect.x + (self.rect.width / 2.0) - (text_width as f32 / 2.0);
        let text_y = self.rect.y + (self.rect.height / 2.0) - (font_size as f32 / 2.0);

        d.draw_text(
            &self.text,
            text_x as i32,
            text_y as i32,
            font_size,
            Color::WHITE,
        );
    }
}
