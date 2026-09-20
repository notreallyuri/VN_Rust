use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use raylib::texture::Image;
use serde_json::Value as Json;
use vn_script::{Event, StoryVm};

use super::{
    AUTO_SLOT, LoadReport, Migrations, SAVE_FORMAT_VERSION, SaveError, SaveFile, SaveMigration,
    SlotInfo, THUMBNAIL_WIDTH, apply, format_migrations, now, summary,
};
use crate::data::rollback::Rollback;
use crate::data::session::LogEntry;
use crate::data::state::GameState;

#[derive(Debug, Clone)]
pub struct Saves {
    dir: PathBuf,
    game: String,
    version: u32,
    migrations: Migrations,
    autosave: bool,
    generation: std::cell::Cell<u64>,
    any_cache: std::cell::Cell<Option<(u64, bool)>>,
}

impl Saves {
    pub fn new(dir: impl Into<PathBuf>, game: impl Into<String>) -> Self {
        Self {
            dir: dir.into(),
            game: game.into(),
            version: 0,
            migrations: Migrations::default(),
            autosave: false,
            generation: std::cell::Cell::new(0),
            any_cache: std::cell::Cell::new(None),
        }
    }

    pub fn with_autosave(mut self, enabled: bool) -> Self {
        self.autosave = enabled;
        self
    }

    pub fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    pub fn with_migrations(mut self, migrations: Migrations) -> Self {
        self.migrations = migrations;
        self
    }

    pub fn with_migration(
        mut self,
        from: u32,
        migration: impl Fn(&mut SaveMigration) -> Result<(), String> + 'static,
    ) -> Self {
        self.migrations.add(from, migration);
        self
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn autosaves(&self) -> bool {
        self.autosave
    }

    pub fn generation(&self) -> u64 {
        self.generation.get()
    }

    fn bump(&self) {
        self.generation.set(self.generation.get() + 1);
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn path(&self, slot: &str) -> Result<PathBuf, SaveError> {
        let valid = !slot.is_empty()
            && slot
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');

        if !valid {
            return Err(SaveError::InvalidSlot(slot.to_string()));
        }

        Ok(self.dir.join(format!("{}.json", slot)))
    }

    pub fn thumbnail_path(&self, slot: &str) -> Result<PathBuf, SaveError> {
        Ok(self.path(slot)?.with_extension("png"))
    }

    pub fn write_thumbnail(&self, slot: &str, image: Option<&Image>) -> Result<(), SaveError> {
        let path = self.thumbnail_path(slot)?;
        let Some(image) = image else {
            return remove_if_present(&path);
        };

        let mut thumbnail = image.clone();
        let height = THUMBNAIL_WIDTH * image.height().max(1) / image.width().max(1);
        thumbnail.resize(THUMBNAIL_WIDTH, height.max(1));

        let temp = path.with_extension("tmp.png");
        thumbnail.export_image(&temp.to_string_lossy());
        if !temp.exists() {
            return Err(SaveError::Io {
                path: temp,
                source: io::Error::other("the image could not be written"),
            });
        }
        fs::rename(&temp, &path).map_err(|source| SaveError::Io { path, source })?;
        self.bump();
        Ok(())
    }

    pub fn capture(&self, story: &StoryVm, state: &GameState) -> Result<SaveFile, SaveError> {
        Ok(SaveFile {
            format_version: SAVE_FORMAT_VERSION,
            game: self.game.clone(),
            game_version: self.version,
            saved_at: now(),
            summary: summary(story),
            story: story.snapshot(),
            state: state.to_json().map_err(SaveError::State)?,
            rollback: Vec::new(),
            log: Vec::new(),
        })
    }

    pub fn save(&self, slot: &str, story: &StoryVm, state: &GameState) -> Result<(), SaveError> {
        let file = self.capture(story, state)?;
        self.write(slot, &file)
    }

    pub fn write(&self, slot: &str, file: &SaveFile) -> Result<(), SaveError> {
        let path = self.path(slot)?;
        let io_error = |path: &Path| {
            let path = path.to_path_buf();
            move |source| SaveError::Io { path, source }
        };

        fs::create_dir_all(&self.dir).map_err(io_error(&self.dir))?;

        let json = serde_json::to_vec_pretty(file).expect("save files serialize");
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, json).map_err(io_error(&temp))?;
        fs::rename(&temp, &path).map_err(io_error(&path))?;
        self.bump();
        Ok(())
    }

    pub fn read(&self, slot: &str) -> Result<SaveFile, SaveError> {
        let path = self.path(slot)?;

        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(SaveError::Empty {
                    slot: slot.to_string(),
                });
            }
            Err(source) => return Err(SaveError::Io { path, source }),
        };

        let mut json: Json =
            serde_json::from_slice(&bytes).map_err(|source| SaveError::Corrupt {
                path: path.clone(),
                source,
            })?;
        self.migrate(&mut json, &path)?;

        let file: SaveFile =
            serde_json::from_value(json).map_err(|source| SaveError::Corrupt { path, source })?;

        if file.game != self.game {
            return Err(SaveError::OtherGame {
                found: file.game,
                expected: self.game.clone(),
            });
        }

        Ok(file)
    }

    pub fn migrate(&self, json: &mut Json, path: &Path) -> Result<(), SaveError> {
        let version_of =
            |json: &Json, key: &str| json.get(key).and_then(Json::as_u64).unwrap_or(0) as u32;

        let format = version_of(json, "format_version");
        if format > SAVE_FORMAT_VERSION {
            return Err(SaveError::NewerFormat {
                found: format,
                supported: SAVE_FORMAT_VERSION,
            });
        }
        let game = version_of(json, "game_version");
        if game > self.version {
            return Err(SaveError::NewerGameVersion {
                found: game,
                supported: self.version,
            });
        }

        let failed = |(from, message)| SaveError::Migration {
            path: path.to_path_buf(),
            from,
            message,
        };
        format_migrations()
            .run(json, format, SAVE_FORMAT_VERSION)
            .map_err(failed)?;
        self.migrations
            .run(json, game, self.version)
            .map_err(failed)?;

        if let Json::Object(file) = json {
            file.insert("format_version".into(), SAVE_FORMAT_VERSION.into());
            file.insert("game_version".into(), self.version.into());
        }
        Ok(())
    }

    pub fn load(
        &self,
        slot: &str,
        story: &mut StoryVm,
        state: &mut GameState,
    ) -> Result<LoadReport, SaveError> {
        let file = self.read(slot)?;
        apply(&file, story, state)
    }

    pub fn delete(&self, slot: &str) -> Result<(), SaveError> {
        remove_if_present(&self.path(slot)?)?;
        remove_if_present(&self.thumbnail_path(slot)?)?;
        self.bump();
        Ok(())
    }

    pub fn slot(&self, slot: &str) -> SlotInfo {
        SlotInfo {
            slot: slot.to_string(),
            save: self.read(slot),
        }
    }

    pub fn has_any(&self) -> bool {
        let generation = self.generation();
        if let Some((seen, any)) = self.any_cache.get()
            && seen == generation
        {
            return any;
        }
        let any = self.latest().is_some();
        self.any_cache.set(Some((generation, any)));
        any
    }

    pub fn latest(&self) -> Option<(String, SaveFile)> {
        let entries = fs::read_dir(&self.dir).ok()?;

        entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.extension()? != "json" {
                    return None;
                }
                let slot = path.file_stem()?.to_str()?.to_string();
                let file = self.read(&slot).ok()?;
                Some((slot, file))
            })
            .max_by_key(|(_, file)| file.saved_at)
    }
}

pub(crate) struct SaveParts<'a> {
    pub story: &'a StoryVm,
    pub state: &'a GameState,
    pub rollback: &'a Rollback,
    pub log: &'a [LogEntry],
    pub thumbnail: Option<&'a Image>,
}

pub(crate) fn save_game(saves: &Saves, slot: &str, parts: SaveParts) -> Result<(), SaveError> {
    let SaveParts {
        story,
        state,
        rollback,
        log,
        thumbnail,
    } = parts;
    let mut file = saves.capture(story, state)?;
    file.rollback = rollback.history();
    file.log = log.to_vec();
    saves.write(slot, &file)?;
    if let Err(e) = saves.write_thumbnail(slot, thumbnail) {
        eprintln!("⚠️ Thumbnail for '{}' not saved: {}", slot, e);
    }
    Ok(())
}

pub(crate) fn autosave(saves: &Saves, parts: SaveParts) {
    if !saves.autosaves() || matches!(parts.story.current(), None | Some(Event::End)) {
        return;
    }
    if let Err(e) = save_game(saves, AUTO_SLOT, parts) {
        eprintln!("⚠️ Autosave failed: {}", e);
    }
}

fn remove_if_present(path: &Path) -> Result<(), SaveError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(SaveError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}
