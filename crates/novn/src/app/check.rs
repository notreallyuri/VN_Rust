use std::path::{Path, PathBuf};

use novn_script::{
    Diagnostic, Instruction, LANG_DIR, Schema, SchemaFile, StoryVm, UI_STRINGS_FILE, UiStrings,
};

use super::{AppError, VnApp};
use crate::data::assets::Assets;
use crate::game::hot_reload::StoryLoader;

impl VnApp {
    pub fn schema(&self) -> Schema {
        let schema = Schema {
            variables: self.variables.clone(),
            characters: self
                .characters
                .iter()
                .map(|(id, c)| (id.to_string(), c.definition()))
                .collect(),
            commands: self.commands.signatures().clone(),
        };
        #[cfg(feature = "character-visuals")]
        let schema = {
            let mut schema = schema;
            self.visuals.extend_schema(&mut schema);
            schema
        };
        schema
    }

    pub fn loader(&self) -> StoryLoader {
        StoryLoader {
            #[cfg(feature = "character-visuals")]
            visuals: self.visuals.clone(),
            assets: self.asset_source(),
            story_dir: PathBuf::from(&self.story_dir),
            schema: self.schema(),
            entry_scene: self.entry_scene.clone(),
            warn_missing_art: self.warn_missing_art,
        }
    }

    pub fn check(&self) -> Result<(StoryVm, Vec<Diagnostic>), AppError> {
        self.loader().load()
    }

    pub fn asset_source(&self) -> Assets {
        let prefer_disk = cfg!(debug_assertions) && self.assets.is_dir();
        if let Some(files) = self.embedded
            && !prefer_disk
        {
            return Assets::Embedded(files);
        }
        if self.assets.is_dir() {
            return Assets::Dir(self.assets.clone());
        }
        let beside_exe = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("assets")))
            .filter(|dir| dir.is_dir());
        Assets::Dir(beside_exe.unwrap_or_else(|| self.assets.clone()))
    }

    pub fn schema_export(&self) -> SchemaFile {
        SchemaFile {
            entry_scene: self.entry_scene.clone(),
            ..SchemaFile::new(&self.title, &self.story_dir, self.schema())
        }
    }

    pub fn schema_path(&self) -> Option<PathBuf> {
        let dir = self.asset_source().dir()?.to_path_buf();
        self.schema_file.as_ref().map(|file| dir.join(file))
    }

    pub fn export_schema(&self) -> Result<Option<PathBuf>, AppError> {
        let Some(path) = self.schema_path() else {
            return Ok(None);
        };
        Ok(self.write_schema(&path)?.then_some(path))
    }

    pub(super) fn write_schema(&self, path: &Path) -> Result<bool, AppError> {
        self.schema_export()
            .write(path)
            .map_err(|source| AppError::Schema {
                path: path.to_path_buf(),
                source,
            })
    }
}

pub(crate) fn missing_art(
    story: &StoryVm,
    assets: &Assets,
    custom_visual: impl Fn(&str, &str) -> bool,
) -> Vec<Diagnostic> {
    let program = story.program();
    let mut seen = std::collections::HashSet::new();

    program
        .instructions
        .iter()
        .enumerate()
        .filter_map(|(index, instruction)| {
            let relative = match instruction {
                Instruction::Show {
                    char_id, img_id, ..
                } => {
                    if custom_visual(char_id, img_id) {
                        return None;
                    }
                    crate::data::resources::character_path(char_id, img_id)
                }
                Instruction::Background { image: Some(image) } => {
                    crate::data::resources::background_path(image)
                }
                Instruction::Music { track: Some(track) }
                    if crate::game::audio::music_path(assets, track).is_none() =>
                {
                    format!("music/{}.ogg", track)
                }
                Instruction::Sound { id }
                    if crate::game::audio::sound_path(assets, id).is_none() =>
                {
                    format!("sounds/{}.ogg", id)
                }
                Instruction::Voice { id }
                    if crate::game::audio::voice_path(assets, id).is_none() =>
                {
                    format!("voice/{}.ogg", id)
                }
                _ => return None,
            };
            let missing = !assets.exists(&relative) && seen.insert(relative.clone());
            missing.then(|| {
                Diagnostic::warning(
                    program.line(index),
                    if relative.ends_with(".png") {
                        format!("missing {} (a placeholder will be drawn)", relative)
                    } else {
                        format!(
                            "missing {} (or .mp3, .wav, .flac); it will be silent",
                            relative
                        )
                    },
                )
                .with_file(program.file(index))
            })
        })
        .collect()
}

impl VnApp {
    pub fn missing_busts(&self) -> Vec<String> {
        if !self.warn_missing_art {
            return Vec::new();
        }
        let assets = self.asset_source();
        self.characters
            .iter()
            .filter_map(|(id, character)| {
                let file = character.bust.as_deref()?;
                let path = crate::data::resources::bust_path(file);
                assets
                    .read(&path)
                    .is_err()
                    .then(|| format!("character '{}': missing {}", id, path))
            })
            .collect()
    }

    pub fn ui_strings(&self) -> UiStrings {
        let mut strings: Vec<String> = Vec::new();
        let mut add = |text: &str| strings.push(text.to_string());

        for text in crate::ui::labels::MESSAGES {
            add(text);
        }
        for text in crate::data::saves::Elapsed::MESSAGES {
            add(text);
        }
        for language in &self.languages {
            add(&language.label);
        }
        for text in self.extra_ui_strings.iter() {
            add(text);
        }

        let start = &self.start;
        for text in [
            Some(&start.prompt),
            start.title.as_ref(),
            start.subtitle.as_ref(),
            start.footer.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            add(text);
        }

        let menu = &self.menu;
        for text in [menu.title.as_ref(), menu.subtitle.as_ref()]
            .into_iter()
            .flatten()
        {
            add(text);
        }
        for item in &menu.items {
            add(&item.label);
            if let Some(tooltip) = &item.tooltip {
                add(tooltip);
            }
        }

        let playing = &self.playing;
        add(&playing.skip_label);
        add(&playing.auto_label);
        add(&playing.end_title);
        add(&playing.end_hint);
        for button in &playing.hud {
            add(&button.label);
            if let Some(tooltip) = &button.tooltip {
                add(tooltip);
            }
        }

        let pause = &self.pause_menu;
        add(&pause.title);
        for item in &pause.items {
            add(&item.label);
            if let Some(tooltip) = &item.tooltip {
                add(tooltip);
            }
        }

        let confirm = &self.confirm_dialog;
        add(&confirm.confirm_label);
        add(&confirm.cancel_label);
        if let Some(message) = &self.close_confirmation {
            add(message);
        }

        let saves = &self.save_menu;
        for text in [
            &saves.save_title,
            &saves.load_title,
            &saves.empty_label,
            &saves.delete_label,
            &saves.back_label,
        ] {
            add(text);
        }

        let settings = &self.settings;
        for text in [
            &settings.title,
            &settings.display_label,
            &settings.windowed_label,
            &settings.fullscreen_label,
            &settings.text_speed_label,
            &settings.music_volume_label,
            &settings.sound_volume_label,
            &settings.voice_volume_label,
            &settings.auto_delay_label,
            &settings.skip_label,
            &settings.skip_seen_label,
            &settings.skip_all_label,
            &settings.language_label,
            &settings.accessibility_label,
            &settings.accessibility_title,
            &settings.open_label,
            &settings.on_label,
            &settings.off_label,
            &settings.reduce_motion_label,
            &settings.text_size_label,
            &settings.text_backdrop_label,
            &settings.text_outline_label,
            &settings.self_voicing_label,
            &settings.sample_text,
            &settings.back_label,
        ] {
            add(text);
        }
        for label in &settings.text_backdrop_levels {
            add(label);
        }
        for (label, _) in &settings.text_speeds {
            add(label);
        }
        for row in crate::screens::settings::SettingsRow::ALL {
            if let Some(tooltip) = settings.row_tooltip(row) {
                add(tooltip);
            }
        }

        let log = &self.log;
        add(&log.title);
        add(&log.empty_label);
        add(&log.back_label);

        let keybinds = &self.keybinds;
        add(&keybinds.title);
        add(&keybinds.back_label);
        add(&keybinds.keys_header);
        add(&keybinds.gamepad_header);
        let sections = keybinds.sections.iter().flatten().chain(&keybinds.extra);
        for section in sections {
            add(&section.title);
            for row in &section.rows {
                add(&row.action);
            }
        }

        add(&self.text_input.hint);

        UiStrings::new(strings)
    }

    pub fn export_ui_strings(&self) -> Result<Option<PathBuf>, AppError> {
        let Some(schema) = self.schema_path() else {
            return Ok(None);
        };
        let root = schema.parent().unwrap_or(Path::new("."));
        let path = root.join(LANG_DIR).join(UI_STRINGS_FILE);
        let strings = self.ui_strings();
        match strings.write(&path) {
            Ok(true) => Ok(Some(path)),
            Ok(false) => Ok(None),
            Err(source) => Err(AppError::Schema { path, source }),
        }
    }
}
