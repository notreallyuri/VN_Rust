use vn_engine::raylib::prelude::*;
use vn_engine::ui::{self, ButtonStyle};
use vn_engine::{DrawContext, GameContext, Screen, ScreenState, TextStyle};

use crate::evidence::{Evidence, describe};
use crate::style;

const ROW_HEIGHT: f32 = 78.0;
const LIST_WIDTH: f32 = 760.0;

pub struct EvidenceScreen {
    heading: TextStyle,
    name: TextStyle,
    detail: TextStyle,
    count: TextStyle,
    back: ButtonStyle,
}

impl EvidenceScreen {
    pub fn new() -> Self {
        Self {
            heading: style::heading(46.0),
            name: style::body(24.0).color(style::PARCHMENT),
            detail: style::label(18.0),
            count: style::label(20.0).color(style::PARCHMENT),
            back: style::button(ButtonStyle::default().size(200.0, 48.0)),
        }
    }

    fn back_rect(&self, screen: Vector2) -> Rectangle {
        Rectangle::new(
            (screen.x - self.back.width) / 2.0,
            screen.y - self.back.height - 40.0,
            self.back.width,
            self.back.height,
        )
    }
}

impl Screen for EvidenceScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(
            &mut ctx,
            Some(&style::background(style::ARCHIVE_BACKGROUND)),
        );
        let back = self.back_rect(ui::screen_size(ctx.rl));
        let key = [
            KeyboardKey::KEY_ESCAPE,
            KeyboardKey::KEY_BACKSPACE,
            KeyboardKey::KEY_E,
        ]
        .into_iter()
        .any(|key| ctx.rl.is_key_pressed(key));

        let nav = ctx.nav.back || ctx.nav.accept;
        (ui::button_clicked(&mut ctx, back, &self.back) || key || nav)
            .then_some(ScreenState::Playing)
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();
        let evidence = ctx.state.get::<Evidence>();

        ui::draw_background(
            d,
            ctx.resources,
            Some(&style::background(style::ARCHIVE_BACKGROUND)),
        );
        d.draw_rectangle(0, 0, screen.x as i32, screen.y as i32, style::BACKDROP);
        ui::draw_text_centered(
            d,
            fonts,
            "Evidence",
            Vector2::new(screen.x / 2.0, 80.0),
            &self.heading,
        );

        let left = (screen.x - LIST_WIDTH) / 2.0;
        let top = 140.0;

        if evidence.is_empty() {
            ui::draw_text_centered(
                d,
                fonts,
                "Nothing has been filed yet.",
                Vector2::new(screen.x / 2.0, top + 40.0),
                &self.detail,
            );
        }

        for (i, (item, count)) in evidence.items().enumerate() {
            let y = top + i as f32 * ROW_HEIGHT;
            let row = Rectangle::new(left, y, LIST_WIDTH, ROW_HEIGHT - 10.0);
            d.draw_rectangle_rounded(row, 0.12, 6, style::PANEL);

            let (name, detail) = describe(item);
            ui::draw_text(
                d,
                fonts,
                name,
                Vector2::new(left + 20.0, y + 10.0),
                &self.name,
            );
            ui::draw_text_wrapped(
                d,
                fonts,
                detail,
                Vector2::new(left + 20.0, y + 40.0),
                LIST_WIDTH - 120.0,
                &self.detail,
            );

            if count > 1 {
                let label = format!("×{}", count);
                let width = fonts.measure(self.count.font, &label, self.count.size).x;
                ui::draw_text(
                    d,
                    fonts,
                    &label,
                    Vector2::new(left + LIST_WIDTH - 20.0 - width, y + 12.0),
                    &self.count,
                );
            }
        }

        ui::Button::new("Back", &self.back)
            .focused(ctx.interactive && ctx.focus_visible)
            .draw(d, ctx, self.back_rect(screen));
    }
}
