use std::path::{Path, PathBuf};

use vn_script::{Diagnostic, Instruction, Schema, SchemaFile, StoryVm};

use super::{AppError, VnApp};
use crate::{Assets, StoryLoader};

impl VnApp {
    pub fn schema(&self) -> Schema {
        Schema {
            variables: self.variables.clone(),
            characters: self
                .characters
                .iter()
                .map(|(id, c)| (id.to_string(), c.definition()))
                .collect(),
            commands: self.commands.signatures().clone(),
        }
    }

    pub fn loader(&self) -> StoryLoader {
        StoryLoader {
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

pub(crate) fn missing_art(story: &StoryVm, assets: &Assets) -> Vec<Diagnostic> {
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
                } => crate::character_path(char_id, img_id),
                Instruction::Background { image: Some(image) } => crate::background_path(image),
                Instruction::Music { track: Some(track) }
                    if crate::music_path(assets, track).is_none() =>
                {
                    format!("music/{}.ogg", track)
                }
                Instruction::Sound { id } if crate::sound_path(assets, id).is_none() => {
                    format!("sounds/{}.ogg", id)
                }
                Instruction::Voice { id } if crate::voice_path(assets, id).is_none() => {
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
