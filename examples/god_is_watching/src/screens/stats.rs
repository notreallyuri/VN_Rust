use vn_engine::raylib::prelude::*;
use vn_engine::script::Value;
use vn_engine::ui;
use vn_engine::{DrawContext, FontRole, GameContext, Overlay, OverlayAction, TextStyle};

use crate::inventory::display_name;

const PANEL_WIDTH: f32 = 420.0;
const ROW_HEIGHT: f32 = 36.0;

pub struct StatsOverlay {
    heading: TextStyle,
    label: TextStyle,
    value: TextStyle,
    hint: TextStyle,
}

impl StatsOverlay {
    pub fn new() -> Self {
        Self {
            heading: TextStyle::new(FontRole::Title, 32.0, Color::RAYWHITE),
            label: TextStyle::new(FontRole::Menu, 20.0, Color::LIGHTGRAY),
            value: TextStyle::new(FontRole::Menu, 20.0, Color::GOLD),
            hint: TextStyle::new(FontRole::Menu, 14.0, Color::GRAY),
        }
    }

    fn rows(ctx: &DrawContext) -> Vec<(String, String)> {
        let mut rows = vec![(
            "Scene".to_string(),
            ctx.story.current_scene().unwrap_or("-").to_string(),
        )];

        let mut variables: Vec<(&String, &Value)> = ctx.story.variables().iter().collect();
        variables.sort_by(|a, b| a.0.cmp(b.0));
        rows.extend(
            variables
                .into_iter()
                .map(|(name, value)| (display_name(name), value.to_string())),
        );

        rows
    }
}

impl Overlay for StatsOverlay {
    fn update(&mut self, ctx: GameContext) -> OverlayAction {
        let close = ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            || ctx.rl.is_key_pressed(KeyboardKey::KEY_TAB)
            || ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
            || ctx.rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE);

        if close {
            OverlayAction::Close
        } else {
            OverlayAction::Stay
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();
        let rows = Self::rows(ctx);

        d.draw_rectangle(
            0,
            0,
            screen.x as i32,
            screen.y as i32,
            Color::new(0, 0, 0, 140),
        );

        let height = 110.0 + rows.len() as f32 * ROW_HEIGHT;
        let panel = Rectangle::new(
            (screen.x - PANEL_WIDTH) / 2.0,
            (screen.y - height) / 2.0,
            PANEL_WIDTH,
            height,
        );
        d.draw_rectangle_rounded(panel, 0.06, 8, Color::new(24, 22, 34, 240));
        d.draw_rectangle_rounded_lines(panel, 0.06, 8, Color::new(255, 255, 255, 40));

        ui::draw_text_centered(
            d,
            fonts,
            "Stats",
            Vector2::new(screen.x / 2.0, panel.y + 36.0),
            &self.heading,
        );

        for (i, (label, value)) in rows.iter().enumerate() {
            let y = panel.y + 70.0 + i as f32 * ROW_HEIGHT;
            ui::draw_text(
                d,
                fonts,
                label,
                Vector2::new(panel.x + 28.0, y),
                &self.label,
            );

            let width = fonts.measure(self.value.font, value, self.value.size).x;
            let x = panel.x + panel.width - 28.0 - width;
            ui::draw_text(d, fonts, value, Vector2::new(x, y), &self.value);
        }

        ui::draw_text_centered(
            d,
            fonts,
            "Click, Tab or Esc to close",
            Vector2::new(screen.x / 2.0, panel.y + panel.height - 20.0),
            &self.hint,
        );
    }
}
