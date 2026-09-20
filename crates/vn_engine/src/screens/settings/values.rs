use super::{AUTO_DELAY_MAX, AUTO_DELAY_MIN, AUTO_DELAY_STEP, SettingsConfig, SettingsRow};
use crate::data::settings::Settings;
use crate::ui;

impl SettingsConfig {
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
        if self.languages.len() > 1 {
            rows.push(SettingsRow::Language);
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
            SettingsRow::Language => {
                let count = self.languages.len();
                let index = self
                    .languages
                    .iter()
                    .position(|language| language.is(settings.language.as_deref()))
                    .unwrap_or(0);
                match count > 1 {
                    true => index as f32 / (count - 1) as f32,
                    false => 0.0,
                }
            }
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
            SettingsRow::Language => {
                let index = ui::slider_step(fraction, self.languages.len());
                if let Some(language) = self.languages.get(index) {
                    settings.language = language.code.clone();
                }
            }
        }
    }

    pub fn step(&self, row: SettingsRow, settings: &mut Settings, delta: i32) {
        match row {
            SettingsRow::Display => settings.fullscreen = !settings.fullscreen,
            SettingsRow::SkipUnseen => settings.skip_unseen = !settings.skip_unseen,
            SettingsRow::Language => {
                let count = self.languages.len();
                if count == 0 {
                    return;
                }
                let current = self
                    .languages
                    .iter()
                    .position(|language| language.is(settings.language.as_deref()))
                    .unwrap_or(0) as i32;
                let next = (current + delta.signum().max(1)).rem_euclid(count as i32);
                settings.language = self.languages[next as usize].code.clone();
            }
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
            SettingsRow::Language => self
                .language_of(settings.language.as_deref())
                .map(|language| language.label.clone())
                .unwrap_or_else(|| settings.language.clone().unwrap_or_default()),
        }
    }

    pub(crate) fn label(&self, row: SettingsRow) -> &str {
        match row {
            SettingsRow::Display => &self.display_label,
            SettingsRow::TextSpeed => &self.text_speed_label,
            SettingsRow::MusicVolume => &self.music_volume_label,
            SettingsRow::SoundVolume => &self.sound_volume_label,
            SettingsRow::VoiceVolume => &self.voice_volume_label,
            SettingsRow::AutoDelay => &self.auto_delay_label,
            SettingsRow::SkipUnseen => &self.skip_label,
            SettingsRow::Language => &self.language_label,
        }
    }

    pub(crate) fn row_tooltip(&self, row: SettingsRow) -> Option<&str> {
        match row {
            SettingsRow::Display => self.display_tooltip.as_deref(),
            SettingsRow::TextSpeed => self.text_speed_tooltip.as_deref(),
            SettingsRow::MusicVolume => self.music_volume_tooltip.as_deref(),
            SettingsRow::SoundVolume => self.sound_volume_tooltip.as_deref(),
            SettingsRow::VoiceVolume => self.voice_volume_tooltip.as_deref(),
            SettingsRow::AutoDelay => self.auto_delay_tooltip.as_deref(),
            SettingsRow::SkipUnseen => self.skip_tooltip.as_deref(),
            SettingsRow::Language => self.language_tooltip.as_deref(),
        }
    }

    pub(crate) fn steps(&self, row: SettingsRow) -> Option<usize> {
        match row {
            SettingsRow::TextSpeed => Some(self.text_speeds.len()),
            SettingsRow::AutoDelay => {
                Some(((AUTO_DELAY_MAX - AUTO_DELAY_MIN) / AUTO_DELAY_STEP) as usize + 1)
            }
            _ => None,
        }
    }
}
