use std::rc::Rc;

use raylib::prelude::*;

use crate::PanelStyle;
use crate::screens::Typewriter;
use crate::ui::{self, Background, ButtonStyle, SliderStyle, TextStyle};
use crate::{
    DrawContext, Focus, FontRole, GameContext, NavInput, Overlay, OverlayAction, Screen,
    ScreenState, Settings,
};

pub const SETTINGS_OVERLAY: &str = "settings";

const PREVIEW_PAUSE: f64 = 1.5;
const ROWS_TOP: f32 = 110.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsRow {
    Display,
    TextSpeed,
    MusicVolume,
    SoundVolume,
    VoiceVolume,
    AutoDelay,
    SkipUnseen,
}

impl SettingsRow {
    pub fn is_slider(self) -> bool {
        !matches!(self, SettingsRow::Display | SettingsRow::SkipUnseen)
    }
}

pub const AUTO_DELAY_MIN: u32 = 500;
pub const AUTO_DELAY_MAX: u32 = 5000;
pub const AUTO_DELAY_STEP: u32 = 500;

#[derive(Clone, Debug)]
pub struct SettingsConfig {
    pub title: String,
    pub title_text: TextStyle,
    pub label_text: TextStyle,
    pub value_button: ButtonStyle,
    pub value_text: TextStyle,
    pub slider: SliderStyle,
    pub focus_panel: PanelStyle,
    pub row_width: f32,
    pub row_spacing: f32,
    pub display_label: String,
    pub windowed_label: String,
    pub fullscreen_label: String,
    pub text_speed_label: String,
    pub text_speeds: Vec<(String, u32)>,
    pub audio_rows: bool,
    pub voice_row: bool,
    pub play_rows: bool,
    pub voice_volume_label: String,
    pub auto_delay_label: String,
    pub skip_label: String,
    pub skip_seen_label: String,
    pub skip_all_label: String,
    pub voice_volume_tooltip: Option<String>,
    pub auto_delay_tooltip: Option<String>,
    pub skip_tooltip: Option<String>,
    pub sample_sound: Option<String>,
    pub music_volume_label: String,
    pub sound_volume_label: String,
    pub volume_step: u32,
    pub display_tooltip: Option<String>,
    pub text_speed_tooltip: Option<String>,
    pub music_volume_tooltip: Option<String>,
    pub sound_volume_tooltip: Option<String>,
    pub sample_text: String,
    pub sample_text_style: TextStyle,
    pub sample_box: PanelStyle,
    pub back_button: ButtonStyle,
    pub back_label: String,
    pub back_keys: Vec<KeyboardKey>,
    pub backdrop: Color,
    pub panel: Option<PanelStyle>,
    pub panel_padding: f32,
    pub background: Option<Background>,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            title: "Settings".to_string(),
            title_text: TextStyle::new(FontRole::Title, 44.0, Color::RAYWHITE),
            label_text: TextStyle::new(FontRole::Menu, 24.0, Color::RAYWHITE),
            value_button: ButtonStyle::default().size(260.0, 40.0).font_size(19.0),
            value_text: TextStyle::new(FontRole::Menu, 18.0, Color::LIGHTGRAY),
            slider: SliderStyle::default(),
            focus_panel: PanelStyle::new(Color::new(255, 255, 255, 22)).roundness(0.2),
            row_width: 600.0,
            row_spacing: 10.0,
            display_label: "Display".to_string(),
            windowed_label: "Windowed".to_string(),
            fullscreen_label: "Fullscreen".to_string(),
            text_speed_label: "Text speed".to_string(),
            text_speeds: vec![
                ("Slow".to_string(), 20),
                ("Normal".to_string(), 40),
                ("Fast".to_string(), 80),
                ("Instant".to_string(), 0),
            ],
            audio_rows: true,
            voice_row: true,
            play_rows: true,
            voice_volume_label: "Voice volume".to_string(),
            auto_delay_label: "Auto-forward".to_string(),
            skip_label: "Skip".to_string(),
            skip_seen_label: "Seen text".to_string(),
            skip_all_label: "All text".to_string(),
            voice_volume_tooltip: Some(
                "Voiced lines. Drag, or hover and press Left/Right".to_string(),
            ),
            auto_delay_tooltip: Some(
                "How long Auto waits after a line (longer lines wait a little more)".to_string(),
            ),
            skip_tooltip: Some(
                "What Skip passes over: only lines you've read, or everything".to_string(),
            ),
            sample_sound: None,
            music_volume_label: "Music volume".to_string(),
            sound_volume_label: "Sound volume".to_string(),
            volume_step: 5,
            display_tooltip: Some("Play in a window, or fill the whole screen".to_string()),
            text_speed_tooltip: Some(
                "How fast lines appear. Clicking while a line types shows all of it".to_string(),
            ),
            music_volume_tooltip: Some(
                "Background music. Drag, or hover and press Left/Right".to_string(),
            ),
            sound_volume_tooltip: Some(
                "Sound effects. Drag, or hover and press Left/Right".to_string(),
            ),
            sample_text: "This is how fast the story's text appears.".to_string(),
            sample_text_style: TextStyle::new(FontRole::Dialogue, 22.0, Color::RAYWHITE),
            sample_box: PanelStyle::new(Color::new(0, 0, 0, 170)),
            back_button: ButtonStyle::default().size(200.0, 48.0),
            back_label: "Back".to_string(),
            back_keys: vec![KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_BACKSPACE],
            backdrop: Color::new(0, 0, 0, 200),
            panel: None,
            panel_padding: 40.0,
            background: None,
        }
    }
}

impl SettingsConfig {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn title_text(mut self, style: TextStyle) -> Self {
        self.title_text = style;
        self
    }

    pub fn label_text(mut self, style: TextStyle) -> Self {
        self.label_text = style;
        self
    }

    pub fn value_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.value_button = style(self.value_button);
        self
    }

    pub fn value_text(mut self, style: TextStyle) -> Self {
        self.value_text = style;
        self
    }

    pub fn slider(mut self, style: impl FnOnce(SliderStyle) -> SliderStyle) -> Self {
        self.slider = style(self.slider);
        self
    }

    pub fn focus_panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.focus_panel = style(self.focus_panel);
        self
    }

    pub fn focus_color(mut self, color: Color) -> Self {
        self.focus_panel.color = color;
        self
    }

    pub fn row_width(mut self, width: f32) -> Self {
        self.row_width = width;
        self
    }

    pub fn row_spacing(mut self, spacing: f32) -> Self {
        self.row_spacing = spacing;
        self
    }

    pub fn text_speeds<S: Into<String>>(
        mut self,
        speeds: impl IntoIterator<Item = (S, u32)>,
    ) -> Self {
        self.text_speeds = speeds
            .into_iter()
            .map(|(label, speed)| (label.into(), speed))
            .collect();
        self
    }

    pub fn audio_rows(mut self, show: bool) -> Self {
        self.audio_rows = show;
        self
    }

    pub fn voice_row(mut self, show: bool) -> Self {
        self.voice_row = show;
        self
    }

    pub fn play_rows(mut self, show: bool) -> Self {
        self.play_rows = show;
        self
    }

    pub fn auto_delay_name(millis: u32) -> String {
        format!("{:.1} s", millis as f32 / 1000.0)
    }

    pub fn sample_sound(mut self, id: impl Into<String>) -> Self {
        self.sample_sound = Some(id.into());
        self
    }

    pub fn volume_step(mut self, percent: u32) -> Self {
        self.volume_step = percent.clamp(1, 100);
        self
    }

    pub fn tooltip(mut self, row: SettingsRow, text: Option<&str>) -> Self {
        let text = text.map(str::to_string);
        match row {
            SettingsRow::Display => self.display_tooltip = text,
            SettingsRow::TextSpeed => self.text_speed_tooltip = text,
            SettingsRow::MusicVolume => self.music_volume_tooltip = text,
            SettingsRow::SoundVolume => self.sound_volume_tooltip = text,
            SettingsRow::VoiceVolume => self.voice_volume_tooltip = text,
            SettingsRow::AutoDelay => self.auto_delay_tooltip = text,
            SettingsRow::SkipUnseen => self.skip_tooltip = text,
        }
        self
    }

    pub fn sample_text(mut self, text: impl Into<String>) -> Self {
        self.sample_text = text.into();
        self
    }

    pub fn sample_text_style(mut self, style: TextStyle) -> Self {
        self.sample_text_style = style;
        self
    }

    pub fn back_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.back_button = style(self.back_button);
        self
    }

    pub fn back_label(mut self, text: impl Into<String>) -> Self {
        self.back_label = text.into();
        self
    }

    pub fn back_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.back_keys = keys.into_iter().collect();
        self
    }

    pub fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = color;
        self
    }

    pub fn sample_box(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.sample_box = style(self.sample_box);
        self
    }

    pub fn sample_box_color(mut self, color: Color) -> Self {
        self.sample_box.color = color;
        self
    }

    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = Some(style(self.panel.unwrap_or_default()));
        self
    }

    pub fn panel_padding(mut self, padding: f32) -> Self {
        self.panel_padding = padding;
        self
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    pub fn text_speed_name(&self, speed: u32) -> String {
        self.text_speeds
            .iter()
            .find(|(_, s)| *s == speed)
            .map(|(label, _)| label.clone())
            .unwrap_or_else(|| format!("{} chars/s", speed))
    }

    pub fn next_text_speed(&self, speed: u32) -> u32 {
        let next = self
            .text_speeds
            .iter()
            .position(|(_, s)| *s == speed)
            .map_or(0, |i| i + 1);
        self.text_speeds
            .get(next % self.text_speeds.len().max(1))
            .map_or(speed, |(_, s)| *s)
    }

    pub fn text_speed_fraction(&self, speed: u32) -> f32 {
        let count = self.text_speeds.len();
        match self.text_speeds.iter().position(|(_, s)| *s == speed) {
            Some(index) if count > 1 => index as f32 / (count - 1) as f32,
            _ => 0.0,
        }
    }

    pub fn text_speed_at(&self, fraction: f32) -> Option<u32> {
        let index = ui::slider_step(fraction, self.text_speeds.len());
        self.text_speeds.get(index).map(|(_, speed)| *speed)
    }

    pub fn volume_at(&self, fraction: f32) -> u32 {
        let step = self.volume_step.max(1) as f32;
        let percent = (fraction.clamp(0.0, 1.0) * 100.0 / step).round() * step;
        (percent as u32).min(100)
    }

    pub fn volume_name(volume: u32) -> String {
        match volume {
            0 => "Off".to_string(),
            v => format!("{}%", v.min(100)),
        }
    }

    pub fn rows(&self) -> Vec<SettingsRow> {
        let mut rows = vec![SettingsRow::Display, SettingsRow::TextSpeed];
        if self.audio_rows {
            rows.extend([SettingsRow::MusicVolume, SettingsRow::SoundVolume]);
            if self.voice_row {
                rows.push(SettingsRow::VoiceVolume);
            }
        }
        if self.play_rows {
            rows.extend([SettingsRow::AutoDelay, SettingsRow::SkipUnseen]);
        }
        rows
    }

    pub fn fraction(&self, row: SettingsRow, settings: &Settings) -> f32 {
        match row {
            SettingsRow::Display => f32::from(u8::from(settings.fullscreen)),
            SettingsRow::TextSpeed => self.text_speed_fraction(settings.text_speed),
            SettingsRow::MusicVolume => settings.music_gain(),
            SettingsRow::SoundVolume => settings.sound_gain(),
            SettingsRow::VoiceVolume => settings.voice_gain(),
            SettingsRow::AutoDelay => {
                let span = (AUTO_DELAY_MAX - AUTO_DELAY_MIN) as f32;
                (settings.auto_delay.clamp(AUTO_DELAY_MIN, AUTO_DELAY_MAX) - AUTO_DELAY_MIN) as f32
                    / span
            }
            SettingsRow::SkipUnseen => f32::from(u8::from(settings.skip_unseen)),
        }
    }

    pub fn set_fraction(&self, row: SettingsRow, settings: &mut Settings, fraction: f32) {
        match row {
            SettingsRow::Display => settings.fullscreen = fraction >= 0.5,
            SettingsRow::TextSpeed => {
                if let Some(speed) = self.text_speed_at(fraction) {
                    settings.text_speed = speed;
                }
            }
            SettingsRow::MusicVolume => settings.music_volume = self.volume_at(fraction),
            SettingsRow::SoundVolume => settings.sound_volume = self.volume_at(fraction),
            SettingsRow::VoiceVolume => settings.voice_volume = self.volume_at(fraction),
            SettingsRow::AutoDelay => {
                let steps = (AUTO_DELAY_MAX - AUTO_DELAY_MIN) / AUTO_DELAY_STEP;
                let step = ui::slider_step(fraction, steps as usize + 1) as u32;
                settings.auto_delay = AUTO_DELAY_MIN + step * AUTO_DELAY_STEP;
            }
            SettingsRow::SkipUnseen => settings.skip_unseen = fraction >= 0.5,
        }
    }

    pub fn step(&self, row: SettingsRow, settings: &mut Settings, delta: i32) {
        match row {
            SettingsRow::Display => settings.fullscreen = !settings.fullscreen,
            SettingsRow::SkipUnseen => settings.skip_unseen = !settings.skip_unseen,
            SettingsRow::AutoDelay => {
                let snapped =
                    (settings.auto_delay + AUTO_DELAY_STEP / 2) / AUTO_DELAY_STEP * AUTO_DELAY_STEP;
                let next = snapped as i64 + delta as i64 * AUTO_DELAY_STEP as i64;
                settings.auto_delay =
                    next.clamp(AUTO_DELAY_MIN as i64, AUTO_DELAY_MAX as i64) as u32;
            }
            SettingsRow::TextSpeed => {
                let count = self.text_speeds.len() as i32;
                let current = self
                    .text_speeds
                    .iter()
                    .position(|(_, s)| *s == settings.text_speed)
                    .map_or(0, |i| i as i32);
                let index = (current + delta).clamp(0, (count - 1).max(0));
                if let Some((_, speed)) = self.text_speeds.get(index as usize) {
                    settings.text_speed = *speed;
                }
            }
            SettingsRow::MusicVolume | SettingsRow::SoundVolume | SettingsRow::VoiceVolume => {
                let volume = match row {
                    SettingsRow::MusicVolume => &mut settings.music_volume,
                    SettingsRow::SoundVolume => &mut settings.sound_volume,
                    _ => &mut settings.voice_volume,
                };
                let step = self.volume_step.max(1) as i32;
                let snapped = (*volume as i32 + step / 2) / step * step;
                *volume = (snapped + delta * step).clamp(0, 100) as u32;
            }
        }
    }

    pub fn value_name(&self, row: SettingsRow, settings: &Settings) -> String {
        match row {
            SettingsRow::Display if settings.fullscreen => self.fullscreen_label.clone(),
            SettingsRow::Display => self.windowed_label.clone(),
            SettingsRow::TextSpeed => self.text_speed_name(settings.text_speed),
            SettingsRow::MusicVolume => Self::volume_name(settings.music_volume),
            SettingsRow::SoundVolume => Self::volume_name(settings.sound_volume),
            SettingsRow::VoiceVolume => Self::volume_name(settings.voice_volume),
            SettingsRow::AutoDelay => Self::auto_delay_name(settings.auto_delay),
            SettingsRow::SkipUnseen if settings.skip_unseen => self.skip_all_label.clone(),
            SettingsRow::SkipUnseen => self.skip_seen_label.clone(),
        }
    }

    fn label(&self, row: SettingsRow) -> &str {
        match row {
            SettingsRow::Display => &self.display_label,
            SettingsRow::TextSpeed => &self.text_speed_label,
            SettingsRow::MusicVolume => &self.music_volume_label,
            SettingsRow::SoundVolume => &self.sound_volume_label,
            SettingsRow::VoiceVolume => &self.voice_volume_label,
            SettingsRow::AutoDelay => &self.auto_delay_label,
            SettingsRow::SkipUnseen => &self.skip_label,
        }
    }

    fn row_tooltip(&self, row: SettingsRow) -> Option<&str> {
        match row {
            SettingsRow::Display => self.display_tooltip.as_deref(),
            SettingsRow::TextSpeed => self.text_speed_tooltip.as_deref(),
            SettingsRow::MusicVolume => self.music_volume_tooltip.as_deref(),
            SettingsRow::SoundVolume => self.sound_volume_tooltip.as_deref(),
            SettingsRow::VoiceVolume => self.voice_volume_tooltip.as_deref(),
            SettingsRow::AutoDelay => self.auto_delay_tooltip.as_deref(),
            SettingsRow::SkipUnseen => self.skip_tooltip.as_deref(),
        }
    }

    fn steps(&self, row: SettingsRow) -> Option<usize> {
        match row {
            SettingsRow::TextSpeed => Some(self.text_speeds.len()),
            SettingsRow::AutoDelay => {
                Some(((AUTO_DELAY_MAX - AUTO_DELAY_MIN) / AUTO_DELAY_STEP) as usize + 1)
            }
            _ => None,
        }
    }

    pub fn control_rects(&self, screen: Vector2) -> Vec<Rectangle> {
        let button = &self.value_button;
        let right = (screen.x + self.row_width) / 2.0;
        (0..self.rows().len())
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

    fn row_rect(&self, control: Rectangle, screen: Vector2) -> Rectangle {
        Rectangle::new(
            (screen.x - self.row_width) / 2.0,
            control.y,
            self.row_width,
            control.height,
        )
    }

    fn sample_rect(&self, screen: Vector2) -> Rectangle {
        let rows = self.control_rects(screen);
        let top = rows.last().map_or(ROWS_TOP, |r| r.y + r.height) + 24.0;
        Rectangle::new(
            (screen.x - self.row_width) / 2.0,
            top,
            self.row_width,
            self.sample_text_style.size * 1.3 * 2.0 + 24.0,
        )
    }

    pub fn panel_rect(&self, screen: Vector2) -> Rectangle {
        let width = self.row_width + self.panel_padding * 2.0;
        Rectangle::new((screen.x - width) / 2.0, 24.0, width, screen.y - 48.0)
    }

    fn back_rect(&self, screen: Vector2) -> Rectangle {
        let style = &self.back_button;
        Rectangle::new(
            (screen.x - style.width) / 2.0,
            screen.y - style.height - 40.0,
            style.width,
            style.height,
        )
    }
}

enum Outcome {
    Stay,
    Back,
}

struct SettingsMenu {
    config: Rc<SettingsConfig>,
    preview: Option<(Typewriter, u32)>,
    visible: usize,
    dragging: Option<SettingsRow>,
    focus: Focus,
}

impl SettingsMenu {
    fn new(config: Rc<SettingsConfig>) -> Self {
        Self {
            config,
            preview: None,
            visible: 0,
            dragging: None,
            focus: Focus::default(),
        }
    }

    fn update(&mut self, ctx: &mut GameContext) -> Outcome {
        let config = Rc::clone(&self.config);
        let screen = ui::screen_size(ctx.rl);

        let back_key = config.back_keys.iter().any(|&k| ctx.rl.is_key_pressed(k));
        let back = config.back_rect(screen);
        if ui::button_clicked(ctx, back, &config.back_button) || back_key || ctx.nav.back {
            return Outcome::Back;
        }

        let rows = config.rows();
        let controls = config.control_rects(screen);

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
            Some(index) if index == rows.len() => return Outcome::Back,
            Some(index) if index < rows.len() && !rows[index].is_slider() => {
                let row = rows[index];
                ctx.settings.update(|s| config.step(row, s, 1));
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
        let mouse = ctx.rl.get_mouse_position();
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
                if ui::button_clicked(ctx, control, &config.value_button) {
                    ctx.settings.update(|s| config.step(row, s, 1));
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
            if pressed && grab.check_collision_point_rec(mouse) {
                self.dragging = Some(row);
            }
        }

        if let Some(row) = self.dragging {
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

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();

        if let Some(panel) = &config.panel {
            panel.draw(d, config.panel_rect(screen));
        }

        ui::draw_text_centered(
            d,
            fonts,
            &config.title,
            Vector2::new(screen.x / 2.0, 70.0),
            &config.title_text,
        );

        let left = (screen.x - config.row_width) / 2.0;
        for (index, (row, control)) in config
            .rows()
            .into_iter()
            .zip(config.control_rects(screen))
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
                config.label(row),
                Vector2::new(left, label_y),
                &config.label_text,
            );

            let value = config.value_name(row, ctx.settings);
            if !row.is_slider() {
                ui::Button::new(&value, &config.value_button)
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

        let sample = config.sample_rect(screen);
        config.sample_box.draw(d, sample);
        ui::draw_text_wrapped_visible(
            d,
            fonts,
            &config.sample_text,
            Vector2::new(sample.x + 16.0, sample.y + 16.0),
            sample.width - 32.0,
            &config.sample_text_style,
            self.visible,
        );

        let back_index = config.rows().len();
        ui::Button::new(&config.back_label, &config.back_button)
            .focused(ctx.shows_focus(&self.focus, back_index))
            .draw(d, ctx, config.back_rect(screen));
    }
}

pub struct SettingsScreen {
    menu: SettingsMenu,
}

impl SettingsScreen {
    pub fn new(config: Rc<SettingsConfig>) -> Self {
        Self {
            menu: SettingsMenu::new(config),
        }
    }
}

impl Screen for SettingsScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, self.menu.config.background.as_ref());

        match self.menu.update(&mut ctx) {
            Outcome::Stay => None,
            Outcome::Back => Some(match ctx.previous {
                Some(state) if *state != ScreenState::Settings => state.clone(),
                _ => ScreenState::MainMenu,
            }),
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        ui::draw_background(d, ctx.resources, self.menu.config.background.as_ref());
        self.menu.draw(d, ctx);
    }
}

pub struct SettingsOverlay {
    menu: SettingsMenu,
}

impl SettingsOverlay {
    pub fn new(config: Rc<SettingsConfig>) -> Self {
        Self {
            menu: SettingsMenu::new(config),
        }
    }
}

impl Overlay for SettingsOverlay {
    fn update(&mut self, mut ctx: GameContext) -> OverlayAction {
        match self.menu.update(&mut ctx) {
            Outcome::Stay => OverlayAction::Stay,
            Outcome::Back => OverlayAction::Close,
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        d.draw_rectangle(
            0,
            0,
            screen.x as i32,
            screen.y as i32,
            self.menu.config.backdrop,
        );
        self.menu.draw(d, ctx);
    }
}
