use raylib::prelude::*;

use super::SettingsConfig;

const ROWS_TOP: f32 = 110.0;
const GAP: f32 = 24.0;
const BAND: f32 = 4.0;
const VIEW_MARGIN: f32 = 28.0;
const MIN_ROWS: f32 = 4.0;

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

    fn rows_height(&self, count: usize) -> f32 {
        let pitch = self.value_button.height + self.row_spacing;
        (count as f32 * pitch - self.row_spacing).max(0.0)
    }

    fn sample_height(&self) -> f32 {
        crate::ui::reading::text(&self.sample_text_style).size * 1.3 * 2.0 + 24.0
    }

    pub fn shows_sample(&self, count: usize, screen: Vector2) -> bool {
        let room = self.back_rect(screen).y - GAP * 2.0 - self.sample_height() - ROWS_TOP;
        let pitch = self.value_button.height + self.row_spacing;
        room >= self.rows_height(count) || room >= MIN_ROWS * pitch - self.row_spacing
    }

    pub fn rows_view(&self, count: usize, screen: Vector2) -> Rectangle {
        let mut bottom = self.back_rect(screen).y - GAP;
        if self.shows_sample(count, screen) {
            bottom -= self.sample_height() + GAP;
        }
        let height = self.rows_height(count).min(bottom - ROWS_TOP).max(0.0);
        Rectangle::new(
            (screen.x - self.row_width) / 2.0 - VIEW_MARGIN,
            ROWS_TOP - BAND,
            self.row_width + VIEW_MARGIN * 2.0,
            height + BAND * 2.0,
        )
    }

    pub fn sample_rect(&self, count: usize, screen: Vector2) -> Rectangle {
        let view = self.rows_view(count, screen);
        Rectangle::new(
            (screen.x - self.row_width) / 2.0,
            view.y + view.height - BAND + GAP,
            self.row_width,
            self.sample_height(),
        )
    }

    pub fn panel_rect(&self, screen: Vector2) -> Rectangle {
        let width = self.row_width + self.panel_padding * 2.0;
        Rectangle::new((screen.x - width) / 2.0, 24.0, width, screen.y - 48.0)
    }

    pub fn back_rect(&self, screen: Vector2) -> Rectangle {
        let style = &self.back_button;
        Rectangle::new(
            (screen.x - style.width) / 2.0,
            screen.y - style.height - 40.0,
            style.width,
            style.height,
        )
    }
}
