use vn_engine::prelude::*;
use vn_engine::raylib::prelude::*;
use vn_engine::ui;
use vn_engine::ui::button::ButtonStyle;

use crate::journal::{Achievements, CaseLedger, ENDINGS, achievement_name, ending_name};
use crate::style;

const LINES: [&str; 5] = [
    "A story built on \"Box 14\" and the Santa Ilde reports",
    "Made with VN_Rust",
    "Fonts: Noto Sans, Noto Serif (SIL OFL). Art generated with ChatGPT",
    "Music: Emma_MA, Tozan, Centurion_of_war, Independent.nu, Bobjt (CC0)",
    "Sounds: Kenney, RPG Audio (CC0). See assets/AUDIO_CREDITS.md",
];

pub struct CreditsScreen {
    heading: TextStyle,
    body: TextStyle,
    section: TextStyle,
    achievement: TextStyle,
    back: ButtonStyle,
}

impl CreditsScreen {
    pub fn new() -> Self {
        Self {
            heading: style::heading(46.0),
            body: style::body(20.0),
            section: style::section(15.0),
            achievement: style::label(19.0).color(style::TEXT),
            back: style::button(ButtonStyle::default().size(200.0, 44.0)),
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

impl Screen for CreditsScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, Some(&style::background(style::TITLE_BACKGROUND)));
        let back = self.back_rect(ui::screen_size(ctx.rl));
        let key = [KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_BACKSPACE]
            .into_iter()
            .any(|key| ctx.rl.is_key_pressed(key));

        let nav = ctx.nav.back || ctx.nav.accept;
        (ui::button::button_clicked(&mut ctx, back, &self.back) || key || nav)
            .then_some(ScreenState::MainMenu)
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();

        ui::draw_background(
            d,
            ctx.resources,
            Some(&style::background(style::TITLE_BACKGROUND)),
        );
        d.draw_rectangle(0, 0, screen.x as i32, screen.y as i32, style::BACKDROP);
        let width = 820.0_f32.min(screen.x - 48.0);
        style::frame(PanelStyle::default()).draw(
            d,
            Rectangle::new((screen.x - width) / 2.0, 24.0, width, screen.y - 48.0),
        );

        ui::draw_text_centered(
            d,
            fonts,
            "God Is Watching",
            Vector2::new(screen.x / 2.0, screen.y * 0.16),
            &self.heading,
        );

        let mut y = screen.y * 0.28;
        for line in LINES {
            ui::draw_text_centered(d, fonts, line, Vector2::new(screen.x / 2.0, y), &self.body);
            y += 36.0;
        }

        let this_time = ctx.story.variable("ending").map(ToString::to_string);
        let ledger = ctx.persistent.get::<CaseLedger>();
        y += 30.0;
        if ledger.closed() > 0 {
            ui::draw_text_centered(
                d,
                fonts,
                &format!("ENDINGS FOUND: {} OF {}", ledger.closed(), ENDINGS.len()),
                Vector2::new(screen.x / 2.0, y),
                &self.section,
            );
            y += 32.0;
            let endings: Vec<String> = ENDINGS
                .iter()
                .map(|&ending| match ledger.has(ending) {
                    true if this_time.as_deref() == Some(ending) => {
                        format!("{} (this time)", ending_name(ending))
                    }
                    true => ending_name(ending).to_string(),
                    false => "???".to_string(),
                })
                .collect();
            ui::draw_text_centered(
                d,
                fonts,
                &endings.join("   ·   "),
                Vector2::new(screen.x / 2.0, y),
                &self.achievement,
            );
            y += 40.0;
        }

        let achievements: Vec<&str> = ctx
            .persistent
            .get::<Achievements>()
            .unlocked()
            .map(achievement_name)
            .collect();
        if !achievements.is_empty() {
            ui::draw_text_centered(
                d,
                fonts,
                "ACHIEVEMENTS",
                Vector2::new(screen.x / 2.0, y),
                &self.section,
            );
            y += 32.0;
            for achievement in achievements {
                ui::draw_text_centered(
                    d,
                    fonts,
                    achievement,
                    Vector2::new(screen.x / 2.0, y),
                    &self.achievement,
                );
                y += 28.0;
            }
        }

        ui::button::Button::new("Back", &self.back)
            .focused(ctx.interactive && ctx.focus_visible)
            .draw(d, ctx, self.back_rect(screen));
    }
}
