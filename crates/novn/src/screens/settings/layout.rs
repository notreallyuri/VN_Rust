use raylib::prelude::*;

use super::SettingsConfig;

const ROWS_TOP: f32 = 110.0;

impl SettingsConfig {
    pub fn control_rects(&self, screen: Vector2) -> Vec<Rectangle> {
        self.control_rects_for(self.rows().len(), screen)
    }

    pub fn control_rects_for(&self, count: usize, screen: Vector2) -> Vec<Rectangle> {
        let button = &self.value_button;
        let right = (screen.x + self.row_width) / 2.0;
        (0..count)
            .map(|i| {
                Rectangle::new(
                    right - button.width,
                    ROWS_TOP + i as f32 * (button.height + self.row_spacing),
                    button.width,
                    button.height,
                )
            })
            .collect()
    }

    pub fn slider_area(&self, control: Rectangle) -> Rectangle {
        let value_width = self.value_text.size * 4.0;
        let inset = self.slider.knob_radius;
        Rectangle::new(
            control.x + inset,
            control.y,
            (control.width - value_width - inset * 2.0).max(0.0),
            control.height,
        )
    }

    pub(crate) fn row_rect(&self, control: Rectangle, screen: Vector2) -> Rectangle {
        Rectangle::new(
            (screen.x - self.row_width) / 2.0,
            control.y,
            self.row_width,
            control.height,
        )
    }

    pub(crate) fn sample_rect(&self, count: usize, screen: Vector2) -> Rectangle {
        let rows = self.control_rects_for(count, screen);
        let top = rows.last().map_or(ROWS_TOP, |r| r.y + r.height) + 24.0;
        Rectangle::new(
            (screen.x - self.row_width) / 2.0,
            top,
            self.row_width,
            crate::ui::reading::text(&self.sample_text_style).size * 1.3 * 2.0 + 24.0,
        )
    }

    pub fn panel_rect(&self, screen: Vector2) -> Rectangle {
        let width = self.row_width + self.panel_padding * 2.0;
        Rectangle::new((screen.x - width) / 2.0, 24.0, width, screen.y - 48.0)
    }

    pub(crate) fn back_rect(&self, screen: Vector2) -> Rectangle {
        let style = &self.back_button;
        Rectangle::new(
            (screen.x - style.width) / 2.0,
            screen.y - style.height - 40.0,
            style.width,
            style.height,
        )
    }
}
