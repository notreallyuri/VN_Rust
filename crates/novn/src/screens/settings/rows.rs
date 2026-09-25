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
    Accessibility,
    ReduceMotion,
    TextSize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SettingsPage {
    #[default]
    Main,
    Accessibility,
}

impl SettingsRow {
    pub const ALL: [SettingsRow; 11] = [
        SettingsRow::Display,
        SettingsRow::TextSpeed,
        SettingsRow::MusicVolume,
        SettingsRow::SoundVolume,
        SettingsRow::VoiceVolume,
        SettingsRow::AutoDelay,
        SettingsRow::SkipUnseen,
        SettingsRow::Language,
        SettingsRow::Accessibility,
        SettingsRow::ReduceMotion,
        SettingsRow::TextSize,
    ];

    pub fn is_slider(self) -> bool {
        !matches!(
            self,
            SettingsRow::Display
                | SettingsRow::SkipUnseen
                | SettingsRow::Language
                | SettingsRow::Accessibility
                | SettingsRow::ReduceMotion
        )
    }
}

pub const AUTO_DELAY_MIN: u32 = 500;
pub const AUTO_DELAY_MAX: u32 = 5000;
pub const AUTO_DELAY_STEP: u32 = 500;
