use vn_engine::prelude::*;
use vn_engine::raylib::prelude::*;
use vn_engine::ui;

use crate::desk::Desk;
use crate::evidence::describe;
use crate::journal::{Journal, note_text};
use crate::style;

const PANEL_WIDTH: f32 = 640.0;
const ROW: f32 = 30.0;

pub struct CaseFileOverlay {
    heading: TextStyle,
    section: TextStyle,
    label: TextStyle,
    value: TextStyle,
    line: TextStyle,
    hint: TextStyle,
}

impl CaseFileOverlay {
    pub fn new() -> Self {
        Self {
            heading: style::heading(34.0),
            section: style::section(15.0),
            label: style::label(19.0),
            value: style::label(19.0).color(style::TEXT),
            line: style::body(17.0),
            hint: style::label(14.0),
        }
    }
}

fn variable(ctx: &DrawContext, name: &str) -> String {
    ctx.story
        .variable(name)
        .map_or_else(|| "-".to_string(), ToString::to_string)
}

impl Overlay for CaseFileOverlay {
    fn update(&mut self, ctx: GameContext) -> OverlayAction {
        let close = ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            || [
                KeyboardKey::KEY_TAB,
                KeyboardKey::KEY_ESCAPE,
                KeyboardKey::KEY_BACKSPACE,
            ]
            .into_iter()
            .any(|key| ctx.rl.is_key_pressed(key))
            || ctx.nav.back;

        if close {
            OverlayAction::Close
        } else {
            OverlayAction::Stay
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();
        let journal = ctx.state.get::<Journal>();

        let facts = [
            ("Approach", variable(ctx, "approach")),
            ("Written in the register", variable(ctx, "verdict")),
            ("The administrator's trust", variable(ctx, "trust")),
            ("The House's suspicion", variable(ctx, "suspicion")),
        ];
        let notes: Vec<&str> = journal.notes().map(note_text).collect();
        let decisions: Vec<&str> = journal.recent_decisions(4).collect();
        let report: Vec<&str> = ctx
            .state
            .get::<Desk>()
            .in_report()
            .map(|item| describe(item).0)
            .collect();

        let height = 120.0
            + facts.len() as f32 * ROW
            + 44.0
            + notes.len().max(1) as f32 * ROW * 1.5
            + 44.0
            + decisions.len().max(1) as f32 * ROW
            + report.len() as f32 * ROW
            + if report.is_empty() { 0.0 } else { 44.0 };
        let panel = Rectangle::new(
            (screen.x - PANEL_WIDTH) / 2.0,
            ((screen.y - height) / 2.0).max(20.0),
            PANEL_WIDTH,
            height,
        );

        d.draw_rectangle(0, 0, screen.x as i32, screen.y as i32, style::BACKDROP);
        style::frame(PanelStyle::default()).draw(d, panel);

        let name = variable(ctx, "player_name");
        ui::draw_text_centered(
            d,
            fonts,
            &format!("Case file: {}", name),
            Vector2::new(screen.x / 2.0, panel.y + 40.0),
            &self.heading,
        );

        let left = panel.x + 32.0;
        let right = panel.x + panel.width - 32.0;
        let mut y = panel.y + 84.0;

        for (label, value) in &facts {
            ui::draw_text(d, fonts, label, Vector2::new(left, y), &self.label);
            let width = fonts.measure(self.value.font, value, self.value.size).x;
            ui::draw_text(d, fonts, value, Vector2::new(right - width, y), &self.value);
            y += ROW;
        }

        y += 14.0;
        ui::draw_text(d, fonts, "NOTES", Vector2::new(left, y), &self.section);
        y += 26.0;
        if notes.is_empty() {
            ui::draw_text(d, fonts, "None yet.", Vector2::new(left, y), &self.hint);
            y += ROW;
        }
        for note in notes {
            y += ui::draw_text_wrapped(
                d,
                fonts,
                note,
                Vector2::new(left, y),
                right - left,
                &self.line,
            ) + 6.0;
        }

        y += 14.0;
        ui::draw_text(
            d,
            fonts,
            "RECENT DECISIONS",
            Vector2::new(left, y),
            &self.section,
        );
        y += 26.0;
        if decisions.is_empty() {
            ui::draw_text(d, fonts, "None yet.", Vector2::new(left, y), &self.hint);
        }
        for decision in decisions {
            ui::draw_text(
                d,
                fonts,
                &format!("· {}", decision),
                Vector2::new(left, y),
                &self.line,
            );
            y += ROW;
        }

        if !report.is_empty() {
            y += 14.0;
            ui::draw_text(
                d,
                fonts,
                "GOING IN THE REPORT",
                Vector2::new(left, y),
                &self.section,
            );
            y += 26.0;
            for item in report {
                ui::draw_text(
                    d,
                    fonts,
                    &format!("· {}", item),
                    Vector2::new(left, y),
                    &self.line,
                );
                y += ROW;
            }
        }

        ui::draw_text_centered(
            d,
            fonts,
            "Click, Tab or Esc to close",
            Vector2::new(screen.x / 2.0, panel.y + panel.height - 18.0),
            &self.hint,
        );
    }
}
