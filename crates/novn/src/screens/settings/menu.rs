use std::rc::Rc;

use raylib::prelude::*;

use super::{SettingsConfig, SettingsPage, SettingsRow};
use crate::context::{DrawContext, GameContext};
use crate::input::navigation::{Focus, NavInput};
use crate::screens::playing::Typewriter;
use crate::ui;

const PREVIEW_PAUSE: f64 = 1.5;

pub(crate) enum Outcome {
    Stay,
    Back,
}

pub(crate) struct SettingsMenu {
    pub(crate) config: Rc<SettingsConfig>,
    preview: Option<(Typewriter, u32)>,
    visible: usize,
    dragging: Option<SettingsRow>,
    focus: Focus,
    page: SettingsPage,
}

impl SettingsMenu {
    pub(crate) fn new(config: Rc<SettingsConfig>) -> Self {
        Self {
            config,
            preview: None,
            visible: 0,
            dragging: None,
            focus: Focus::default(),
            page: SettingsPage::Main,
        }
    }

    pub(crate) fn update(&mut self, ctx: &mut GameContext) -> Outcome {
        let language = ctx.settings.values.language.clone();
        let outcome = self.changes(ctx);
        if ctx.settings.values.language != language {
            ctx.apply_language();
        }
        outcome
    }

    fn changes(&mut self, ctx: &mut GameContext) -> Outcome {
        let config = Rc::clone(&self.config);
        let screen = ui::screen_size(ctx.rl);

        let back_key = config.back_keys.iter().any(|&k| ctx.rl.is_key_pressed(k));
        let back = config.back_rect(screen);
        if ui::button::button_clicked(ctx, back, &config.back_button) || back_key || ctx.nav.back {
            return self.back();
        }

        let rows = config.page_rows(self.page);
        let controls = config.control_rects_for(rows.len(), screen);

        let mut targets: Vec<Rectangle> = controls
            .iter()
            .map(|&control| config.row_rect(control, screen))
            .collect();
        targets.push(back);
        let pointed = targets
            .iter()
            .position(|rect| ui::is_hovered(ctx.rl, *rect));
        let horizontal = ctx.nav.horizontal();
        let vertical_only = NavInput {
            left: false,
            right: false,
            ..ctx.nav
        };
        match self.focus.update(&vertical_only, &targets, &[], pointed) {
            Some(index) if index == rows.len() => return self.back(),
            Some(index) if index < rows.len() && !rows[index].is_slider() => {
                let row = rows[index];
                if self.activate(row, ctx, &config) {
                    return Outcome::Stay;
                }
            }
            _ => {}
        }
        let mut sample = false;
        if horizontal != 0
            && let Some(&row) = self.focus.index().and_then(|i| rows.get(i))
        {
            ctx.settings.update(|s| config.step(row, s, horizontal));
            sample |= row == SettingsRow::SoundVolume;
        }
        let mouse = crate::frame::viewport::mouse_position(ctx.rl);
        let pressed = ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let held = ctx.rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);

        for (&row, &control) in rows.iter().zip(&controls) {
            let row_rect = config.row_rect(control, screen);
            if let Some(text) = config.row_tooltip(row)
                && self.dragging.is_none()
            {
                ctx.tooltip(row_rect, text);
            }

            if !row.is_slider() {
                if ui::button::button_clicked(ctx, control, &config.value_button)
                    && self.activate(row, ctx, &config)
                {
                    return Outcome::Stay;
                }
                continue;
            }

            let area = config.slider_area(control);
            let grab = Rectangle::new(
                area.x - config.slider.knob_radius,
                area.y,
                area.width + config.slider.knob_radius * 2.0,
                area.height,
            );
            if grab.check_collision_point_rec(mouse) && self.dragging.is_none() {
                ctx.cursor(crate::ui::cursor::CursorKind::Grab);
            }
            if pressed && grab.check_collision_point_rec(mouse) {
                self.dragging = Some(row);
            }
        }

        if let Some(row) = self.dragging {
            ctx.cursor(crate::ui::cursor::CursorKind::Grabbing);
            let index = rows.iter().position(|&r| r == row);
            if let Some(control) = index.map(|i| controls[i]) {
                let fraction = ui::slider_fraction(config.slider_area(control), mouse.x);
                ctx.settings
                    .update(|s| config.set_fraction(row, s, fraction));
            }
            if !held {
                self.dragging = None;
                sample |= row == SettingsRow::SoundVolume;
            }
        }

        if sample && let Some(id) = &config.sample_sound {
            let values = &ctx.settings.values;
            ctx.audio
                .set_volumes(values.music_gain(), values.sound_gain());
            ctx.play_sound(id);
        }

        self.animate_preview(ctx.rl.get_time(), ctx.settings.values.text_speed);
        Outcome::Stay
    }

    fn back(&mut self) -> Outcome {
        match self.page {
            SettingsPage::Main => Outcome::Back,
            SettingsPage::Accessibility => {
                self.page = SettingsPage::Main;
                let at = self
                    .config
                    .rows()
                    .iter()
                    .position(|&row| row == SettingsRow::Accessibility);
                self.focus.set(at);
                self.dragging = None;
                Outcome::Stay
            }
        }
    }

    fn activate(
        &mut self,
        row: SettingsRow,
        ctx: &mut GameContext,
        config: &SettingsConfig,
    ) -> bool {
        if row == SettingsRow::Accessibility {
            self.page = SettingsPage::Accessibility;
            self.focus = Focus::default();
            self.dragging = None;
            return true;
        }
        ctx.settings.update(|s| config.step(row, s, 1));
        false
    }

    fn animate_preview(&mut self, now: f64, speed: u32) {
        let text = &self.config.sample_text;
        let restart = match &self.preview {
            Some((_, shown_speed)) if *shown_speed != speed => true,
            Some((typewriter, _)) => typewriter.is_done(now - PREVIEW_PAUSE) && speed != 0,
            None => true,
        };

        if restart {
            self.preview = Some((Typewriter::start(text, speed, now), speed));
        }
        self.visible = self
            .preview
            .as_ref()
            .map_or(0, |(typewriter, _)| typewriter.visible(now));
    }

    pub(crate) fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();

        if let Some(panel) = &config.panel {
            panel.draw(d, config.panel_rect(screen));
        }

        ui::draw_text_centered(
            d,
            fonts,
            ctx.label(config.title_of(self.page)),
            Vector2::new(screen.x / 2.0, 70.0),
            &config.title_text,
        );

        let left = (screen.x - config.row_width) / 2.0;
        let rows = config.page_rows(self.page);
        let count = rows.len();
        for (index, (row, control)) in rows
            .into_iter()
            .zip(config.control_rects_for(count, screen))
            .enumerate()
        {
            let focused = ctx.shows_focus(&self.focus, index);
            if focused {
                let row_rect = config.row_rect(control, screen);
                let band = Rectangle::new(
                    row_rect.x - 12.0,
                    row_rect.y - 4.0,
                    row_rect.width + 24.0,
                    row_rect.height + 8.0,
                );
                config.focus_panel.draw(d, band);
            }
            let label_y = control.y + (control.height - config.label_text.size) / 2.0;
            ui::draw_text(
                d,
                fonts,
                ctx.label(config.label(row)),
                Vector2::new(left, label_y),
                &config.label_text,
            );

            let value = config.value_name(row, ctx.settings);
            let value = ctx.label(&value).to_string();
            if !row.is_slider() {
                ui::button::Button::new(&value, &config.value_button)
                    .focused(focused)
                    .draw(d, ctx, control);
                continue;
            }

            let area = config.slider_area(control);
            ui::draw_slider(
                d,
                area,
                config.fraction(row, ctx.settings),
                config.steps(row),
                focused || self.dragging == Some(row) || ctx.pointer_over(d, area),
                &config.slider,
            );

            let style = &config.value_text;
            let width = fonts.measure(style.font, &value, style.size).x;
            let value_x = control.x + control.width - width;
            let value_y = control.y + (control.height - style.size) / 2.0;
            ui::draw_text(d, fonts, &value, Vector2::new(value_x, value_y), style);
        }

        let sample = config.sample_rect(count, screen);
        config.sample_box.draw(d, sample);
        ui::draw_text_wrapped_visible(
            d,
            fonts,
            ctx.label(&config.sample_text),
            Vector2::new(sample.x + 16.0, sample.y + 16.0),
            sample.width - 32.0,
            &config.sample_text_style,
            self.visible,
        );

        let back_index = count;
        ui::button::Button::new(ctx.label(&config.back_label), &config.back_button)
            .focused(ctx.shows_focus(&self.focus, back_index))
            .draw(d, ctx, config.back_rect(screen));
    }
}
