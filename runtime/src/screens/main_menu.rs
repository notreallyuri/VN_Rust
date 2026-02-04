use raylib::prelude::*;
use vn_core::{
    runtime::{
        ResourceManager,
        types::{GameContext, Screen, ScreenState},
    },
    script::provider::StoryProvider,
};

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

pub struct MainMenuScreen {
    buttons: Vec<Button>,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        let mut buttons = Vec::new();

        let center_x = 1280.0 / 2.0 - 100.0;
        let primary_color = Color::new(40, 40, 60, 255);

        buttons.push(Button::new(
            center_x,
            300.0,
            200.0,
            50.0,
            "New Game",
            primary_color,
        ));
        buttons.push(Button::new(
            center_x,
            370.0,
            200.0,
            50.0,
            "Settings",
            primary_color,
        ));
        buttons.push(Button::new(
            center_x,
            440.0,
            200.0,
            50.0,
            "Exit",
            Color::new(80, 30, 30, 255),
        ));

        Self { buttons }
    }
}

impl Screen for MainMenuScreen {
    fn update(&mut self, ctx: GameContext) -> Option<ScreenState> {
        if self.buttons[0].is_clicked(ctx.rl) {
            return Some(ScreenState::Playing);
        }
        if self.buttons[1].is_clicked(ctx.rl) {
            return Some(ScreenState::Settings("display".to_string()));
        }
        if self.buttons[2].is_clicked(ctx.rl) {
            std::process::exit(0);
        }

        None
    }

    fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        _resources: &mut ResourceManager,
        _story: &StoryProvider,
    ) {
        d.draw_text("God Is Watching", 480, 150, 60, Color::RAYWHITE);

        for b in &self.buttons {
            b.draw(d);
        }
    }
}
