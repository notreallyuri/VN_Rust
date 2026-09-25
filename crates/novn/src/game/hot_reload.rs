use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use novn_script::{Diagnostic, RestoreOutcome, Schema, StoryVm, VmError, story_files};

use crate::app::AppError;
use crate::data::assets::Assets;
use crate::data::rollback::Rollback;

pub const HOT_RELOAD_INTERVAL: f64 = 0.5;

#[derive(Clone, Debug)]
pub struct StoryLoader {
    #[cfg(feature = "character-visuals")]
    pub visuals: crate::game::visuals::VisualRegistry,
    pub assets: Assets,
    pub story_dir: PathBuf,
    pub schema: Schema,
    pub entry_scene: Option<String>,
    pub warn_missing_art: bool,
}

impl StoryLoader {
    pub fn path(&self) -> PathBuf {
        PathBuf::from(self.assets.describe(&self.story_dir.to_string_lossy()))
    }

    pub fn watch_dir(&self) -> Option<PathBuf> {
        self.assets.dir().map(|dir| dir.join(&self.story_dir))
    }

    pub fn sources(&self) -> io::Result<Vec<(String, String)>> {
        let story_dir = self.story_dir.to_string_lossy();
        self.assets
            .files_under(&story_dir)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", self.path().display(), e)))?
            .into_iter()
            .filter(|file| file.ends_with(".story"))
            .map(|file| {
                let source = self.assets.read_to_string(&file)?;
                Ok((self.assets.describe(&file), source))
            })
            .collect()
    }

    pub fn load(&self) -> Result<(StoryVm, Vec<Diagnostic>), AppError> {
        #[cfg(feature = "character-visuals")]
        self.visuals
            .validate(&self.schema, &self.assets)
            .map_err(AppError::Visual)?;
        let path = self.path();
        let sources = self.sources().map_err(|source| AppError::Story {
            path: path.clone(),
            source,
        })?;
        let mut story = StoryVm::from_program(novn_script::compile_sources(sources));

        if story.program().files.is_empty() {
            return Err(AppError::Story {
                source: io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("no .story files in {}", path.display()),
                ),
                path,
            });
        }

        let mut diagnostics = story.prepare(self.schema.clone(), self.entry_scene.as_deref());
        if self.warn_missing_art {
            #[cfg(feature = "character-visuals")]
            let custom_visual = |id: &str, image: &str| self.visuals.handles(id, image);
            #[cfg(not(feature = "character-visuals"))]
            let custom_visual = |_: &str, _: &str| false;
            diagnostics.extend(crate::app::missing_art(&story, &self.assets, custom_visual));
        }

        let (errors, warnings): (Vec<_>, Vec<_>) =
            diagnostics.into_iter().partition(Diagnostic::is_error);

        if errors.is_empty() {
            Ok((story, warnings))
        } else {
            Err(AppError::Script { path, errors })
        }
    }
}

#[derive(Debug)]
pub struct StoryWatcher {
    dir: PathBuf,
    stamps: Vec<(PathBuf, Option<SystemTime>)>,
    last_check: Option<f64>,
}

impl StoryWatcher {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        let stamps = stamps(&dir);
        Self {
            dir,
            stamps,
            last_check: None,
        }
    }

    pub fn poll(&mut self, now: f64) -> bool {
        if self
            .last_check
            .is_some_and(|last| now - last < HOT_RELOAD_INTERVAL)
        {
            return false;
        }
        self.last_check = Some(now);
        self.changed()
    }

    pub fn changed(&mut self) -> bool {
        let current = stamps(&self.dir);
        if current == self.stamps {
            return false;
        }
        self.stamps = current;
        true
    }
}

fn stamps(dir: &Path) -> Vec<(PathBuf, Option<SystemTime>)> {
    story_files(dir)
        .unwrap_or_default()
        .into_iter()
        .map(|path| {
            let modified = path.metadata().and_then(|m| m.modified()).ok();
            (path, modified)
        })
        .collect()
}

pub fn swap_story(
    current: &mut StoryVm,
    mut fresh: StoryVm,
    rollback: &mut Rollback,
) -> Result<RestoreOutcome, VmError> {
    fresh.set_catalog(current.catalog().cloned());
    let outcome = fresh.restore(&current.snapshot())?;
    *current = fresh;
    match outcome {
        RestoreOutcome::Exact => rollback.retain_valid(current),
        RestoreOutcome::SceneRestarted { .. } => rollback.clear(),
    }
    Ok(outcome)
}
