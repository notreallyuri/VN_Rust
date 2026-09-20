#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsRow {
    Display,
    TextSpeed,
    MusicVolume,
    SoundVolume,
    VoiceVolume,
    AutoDelay,
    SkipUnseen,
    Language,
}

impl SettingsRow {
    pub fn is_slider(self) -> bool {
        !matches!(
            self,
            SettingsRow::Display | SettingsRow::SkipUnseen | SettingsRow::Language
        )
    }
}

pub const AUTO_DELAY_MIN: u32 = 500;
pub const AUTO_DELAY_MAX: u32 = 5000;
pub const AUTO_DELAY_STEP: u32 = 500;
