use vn_engine::raylib::prelude::*;
use vn_engine::ui::{self, ButtonStyle};
use vn_engine::{DrawContext, FontRole, GameContext, Screen, ScreenState, TextStyle};

use crate::inventory::{Inventory, display_name};

const ROW_HEIGHT: f32 = 56.0;
const LIST_WIDTH: f32 = 560.0;

pub struct InventoryScreen {
    heading: TextStyle,
    item: TextStyle,
    count: TextStyle,
    empty: TextStyle,
    back: ButtonStyle,
}

impl InventoryScreen {
    pub fn new() -> Self {
        Self {
            heading: TextStyle::new(FontRole::Title, 48.0, Color::RAYWHITE),
            item: TextStyle::new(FontRole::Dialogue, 24.0, Color::RAYWHITE),
            count: TextStyle::new(FontRole::Menu, 22.0, Color::GOLD),
            empty: TextStyle::new(FontRole::Dialogue, 22.0, Color::GRAY),
            back: ButtonStyle::default().size(200.0, 48.0),
        }
    }

    fn back_rect(&self, screen: Vector2) -> Rectangle {
        Rectangle::new(
            (screen.x - self.back.width) / 2.0,
            screen.y - self.back.height - 48.0,
            self.back.width,
            self.back.height,
        )
    }
}

impl Screen for InventoryScreen {
    fn update(&mut self, ctx: GameContext) -> Option<ScreenState> {
        let back = self.back_rect(ui::screen_size(ctx.rl));
        let key = ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
            || ctx.rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
            || ctx.rl.is_key_pressed(KeyboardKey::KEY_I);

        (ui::is_clicked(ctx.rl, back) || key).then_some(ScreenState::Playing)
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();
        let inventory = ctx.state.get::<Inventory>();

        d.clear_background(Color::new(14, 12, 18, 255));
        ui::draw_text_centered(
            d,
            fonts,
            "Inventory",
            Vector2::new(screen.x / 2.0, 90.0),
            &self.heading,
        );

        let left = (screen.x - LIST_WIDTH) / 2.0;
        let top = 160.0;

        if inventory.is_empty() {
            ui::draw_text_centered(
                d,
                fonts,
                "Nothing yet.",
                Vector2::new(screen.x / 2.0, top + ROW_HEIGHT),
                &self.empty,
            );
        }

        for (i, (item, count)) in inventory.items().enumerate() {
            let y = top + i as f32 * ROW_HEIGHT;
            let row = Rectangle::new(left, y, LIST_WIDTH, ROW_HEIGHT - 8.0);
            d.draw_rectangle_rec(row, Color::new(255, 255, 255, 12));

            let text_y = y + (ROW_HEIGHT - 8.0 - self.item.size) / 2.0;
            ui::draw_text(
                d,
                fonts,
                &display_name(item),
                Vector2::new(left + 20.0, text_y),
                &self.item,
            );

            let label = format!("×{}", count);
            let width = fonts.measure(self.count.font, &label, self.count.size).x;
            ui::draw_text(
                d,
                fonts,
                &label,
                Vector2::new(left + LIST_WIDTH - 20.0 - width, text_y),
                &self.count,
            );
        }

        ui::draw_button(d, fonts, self.back_rect(screen), "Back", &self.back);
    }
}
