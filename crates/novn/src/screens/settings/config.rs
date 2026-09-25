use raylib::prelude::*;

use super::SettingsRow;
use crate::game::language::Language;
use crate::ui::button::ButtonStyle;
use crate::ui::fonts::FontRole;
use crate::ui::shape::PanelStyle;
use crate::ui::theme::Theme;
use crate::ui::{Background, SliderStyle, TextStyle};

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
    pub accessibility_rows: bool,
    pub accessibility_label: String,
    pub accessibility_title: String,
    pub accessibility_tooltip: Option<String>,
    pub open_label: String,
    pub on_label: String,
    pub off_label: String,
    pub reduce_motion_label: String,
    pub reduce_motion_tooltip: Option<String>,
    pub text_size_label: String,
    pub text_size_tooltip: Option<String>,
    pub text_backdrop_label: String,
    pub text_backdrop_levels: [String; 3],
    pub text_backdrop_tooltip: Option<String>,
    pub text_outline_label: String,
    pub text_outline_tooltip: Option<String>,
    pub self_voicing_label: String,
    pub self_voicing_tooltip: Option<String>,
    pub self_voicing_row: bool,
    pub voice_volume_label: String,
    pub auto_delay_label: String,
    pub skip_label: String,
    pub skip_seen_label: String,
    pub skip_all_label: String,
    pub language_label: String,
    pub language_tooltip: Option<String>,
    pub languages: Vec<Language>,
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
            accessibility_rows: true,
            accessibility_label: "Accessibility".to_string(),
            accessibility_title: "Accessibility".to_string(),
            accessibility_tooltip: Some("Motion, text size and reading aids".to_string()),
            open_label: "Open".to_string(),
            on_label: "On".to_string(),
            off_label: "Off".to_string(),
            reduce_motion_label: "Reduce motion".to_string(),
            reduce_motion_tooltip: Some(
                "No slides, shaking, flashes or drifting scenery. Fades stay".to_string(),
            ),
            text_size_label: "Text size".to_string(),
            text_size_tooltip: Some(
                "Dialogue, choices and the log. The box grows to fit".to_string(),
            ),
            text_backdrop_label: "Text backdrop".to_string(),
            text_backdrop_levels: [
                "As designed".to_string(),
                "Stronger".to_string(),
                "Solid".to_string(),
            ],
            text_backdrop_tooltip: Some(
                "Makes the dialogue box less see-through. Its colour stays the same".to_string(),
            ),
            text_outline_label: "Text outline".to_string(),
            text_outline_tooltip: Some(
                "A thin edge around dialogue, for text over a busy background".to_string(),
            ),
            self_voicing_label: "Self-voicing".to_string(),
            self_voicing_tooltip: Some(
                "Reads each line and choice aloud as it appears".to_string(),
            ),
            self_voicing_row: false,
            voice_volume_label: "Voice volume".to_string(),
            auto_delay_label: "Auto-forward".to_string(),
            skip_label: "Skip".to_string(),
            skip_seen_label: "Seen text".to_string(),
            skip_all_label: "All text".to_string(),
            language_label: "Language".to_string(),
            language_tooltip: Some(
                "The language the story is read in. Lines with no translation stay as written"
                    .to_string(),
            ),
            languages: Vec::new(),
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

    pub fn language_label(mut self, label: impl Into<String>) -> Self {
        self.language_label = label.into();
        self
    }

    pub fn language_of(&self, code: Option<&str>) -> Option<&Language> {
        self.languages.iter().find(|language| language.is(code))
    }

    pub fn voice_row(mut self, show: bool) -> Self {
        self.voice_row = show;
        self
    }

    pub fn play_rows(mut self, show: bool) -> Self {
        self.play_rows = show;
        self
    }

    pub fn accessibility_rows(mut self, show: bool) -> Self {
        self.accessibility_rows = show;
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
            SettingsRow::Language => self.language_tooltip = text,
            SettingsRow::Accessibility => self.accessibility_tooltip = text,
            SettingsRow::ReduceMotion => self.reduce_motion_tooltip = text,
            SettingsRow::TextSize => self.text_size_tooltip = text,
            SettingsRow::TextBackdrop => self.text_backdrop_tooltip = text,
            SettingsRow::TextOutline => self.text_outline_tooltip = text,
            SettingsRow::SelfVoicing => self.self_voicing_tooltip = text,
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
}

impl SettingsConfig {
    pub fn themed(mut self, theme: &Theme) -> Self {
        self.title_text = theme.text(&theme.title, self.title_text);
        self.label_text = theme.text(&theme.label, self.label_text);
        self.value_text = theme.text(&theme.label, self.value_text);
        self.value_button = theme.buttons(&theme.button, self.value_button);
        self.back_button = theme.buttons(&theme.button, self.back_button);
        self.sample_box = theme.surface(&theme.inset, self.sample_box);
        self.panel = self.panel.map(|panel| theme.surface(&theme.panel, panel));
        self.backdrop = theme.dim(self.backdrop);
        self.background = theme.behind(self.background);
        self
    }
}
