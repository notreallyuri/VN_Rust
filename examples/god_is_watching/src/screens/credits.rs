use vn_engine::raylib::prelude::*;
use vn_engine::ui::{self, ButtonStyle};
use vn_engine::{DrawContext, FontRole, GameContext, Screen, ScreenState, TextStyle};

const LINES: [&str; 3] = [
    "Story adapted from \"Box 14\"",
    "Built with VN_Rust",
    "Fonts: Noto Sans, Noto Serif (SIL OFL)",
];

pub struct CreditsScreen {
    heading: TextStyle,
    body: TextStyle,
    back: ButtonStyle,
}

impl CreditsScreen {
    pub fn new() -> Self {
        Self {
            heading: TextStyle::new(FontRole::Title, 48.0, Color::RAYWHITE),
            body: TextStyle::new(FontRole::Dialogue, 24.0, Color::LIGHTGRAY),
            back: ButtonStyle::default().size(200.0, 48.0),
        }
    }

    fn back_rect(&self, screen: Vector2) -> Rectangle {
        Rectangle::new(
            (screen.x - self.back.width) / 2.0,
            screen.y * 0.75,
            self.back.width,
            self.back.height,
        )
    }
}

impl Screen for CreditsScreen {
    fn update(&mut self, ctx: GameContext) -> Option<ScreenState> {
        let back = self.back_rect(ui::screen_size(ctx.rl));
        let escape = ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
            || ctx.rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE);

        (ui::is_clicked(ctx.rl, back) || escape).then_some(ScreenState::MainMenu)
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();

        ui::draw_text_centered(
            d,
            fonts,
            "Credits",
            Vector2::new(screen.x / 2.0, screen.y * 0.25),
            &self.heading,
        );

        for (i, line) in LINES.iter().enumerate() {
            let y = screen.y * 0.4 + i as f32 * 40.0;
            ui::draw_text_centered(d, fonts, line, Vector2::new(screen.x / 2.0, y), &self.body);
        }

        ui::draw_button(d, fonts, self.back_rect(screen), "Back", &self.back);
    }
}
