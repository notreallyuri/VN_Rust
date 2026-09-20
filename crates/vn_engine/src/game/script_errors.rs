use std::path::Path;

use raylib::prelude::*;

use crate::app::AppError;
use crate::ui;
use crate::ui::TextStyle;
use crate::ui::fonts::{FontRole, Fonts};

pub const SCRIPT_ERRORS_KEY: KeyboardKey = KeyboardKey::KEY_F2;

const MARGIN: f32 = 16.0;
const PADDING: f32 = 14.0;
const MAX_HEIGHT: f32 = 0.6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptErrors {
    pub title: String,
    pub lines: Vec<String>,
    pub collapsed: bool,
}

impl ScriptErrors {
    pub fn from_error(error: &AppError, story_dir: &Path) -> Self {
        let lines = match error {
            AppError::Script { errors, .. } => errors
                .iter()
                .map(|diagnostic| {
                    let mut diagnostic = diagnostic.clone();
                    diagnostic.file = diagnostic
                        .file
                        .map(|file| relative(&file, story_dir).to_string());
                    diagnostic.to_string()
                })
                .collect(),
            other => vec![other.to_string()],
        };

        let count = lines.len();
        Self {
            title: format!(
                "Story not reloaded: {} error{} (still running the previous version)",
                count,
                if count == 1 { "" } else { "s" }
            ),
            lines,
            collapsed: false,
        }
    }

    pub fn toggle(&mut self) {
        self.collapsed = !self.collapsed;
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, fonts: &Fonts) {
        let title_style = TextStyle::new(FontRole::Menu, 18.0, Color::new(255, 190, 170, 255));
        let line_style = TextStyle::new(FontRole::Menu, 16.0, Color::RAYWHITE);
        let hint_style = TextStyle::new(FontRole::Menu, 14.0, Color::new(200, 170, 170, 255));
        let screen = ui::screen_size(d);
        let width = screen.x - MARGIN * 2.0;
        let inner = width - PADDING * 2.0;

        let hint = if self.collapsed {
            "F2 to show"
        } else {
            "Fix the file and save to reload  ·  F2 to hide"
        };

        let mut wrapped = Vec::new();
        if !self.collapsed {
            for line in &self.lines {
                wrapped.extend(fonts.wrap(line_style.font, line, line_style.size, inner));
            }
        }

        let line_height = line_style.size * 1.3;
        let fixed = PADDING * 2.0 + title_style.size * 1.4 + hint_style.size * 1.4;
        let room = ((screen.y * MAX_HEIGHT - fixed) / line_height).max(1.0) as usize;
        let hidden = wrapped.len().saturating_sub(room);
        if hidden > 0 {
            wrapped.truncate(room.saturating_sub(1));
            wrapped.push(format!(
                "... and {} more lines (see the console)",
                hidden + 1
            ));
        }

        let height = fixed + wrapped.len() as f32 * line_height;
        let panel = Rectangle::new(MARGIN, MARGIN, width, height);
        d.draw_rectangle_rec(panel, Color::new(60, 12, 16, 235));
        d.draw_rectangle_lines_ex(panel, 2.0, Color::new(200, 70, 70, 255));

        let x = panel.x + PADDING;
        let mut y = panel.y + PADDING;
        ui::draw_text(d, fonts, &self.title, Vector2::new(x, y), &title_style);
        y += title_style.size * 1.4;

        for line in &wrapped {
            ui::draw_text(d, fonts, line, Vector2::new(x, y), &line_style);
            y += line_height;
        }

        ui::draw_text(d, fonts, hint, Vector2::new(x, y), &hint_style);
    }
}

fn relative<'a>(file: &'a str, dir: &Path) -> &'a str {
    Path::new(file)
        .strip_prefix(dir)
        .ok()
        .and_then(Path::to_str)
        .unwrap_or(file)
}
