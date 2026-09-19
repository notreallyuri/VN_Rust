use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use vn_script::{
    Diagnostic, Program, SCHEMA_FILE_NAME, SchemaFile, StoryVm, compile_sources, story_files,
};

#[derive(Clone, Debug)]
pub struct Project {
    pub stories: PathBuf,
    pub single: Option<PathBuf>,
    pub schema: Option<SchemaFile>,
    pub assets: Option<PathBuf>,
}

pub struct Analysis {
    pub program: Program,
    pub diagnostics: Vec<Diagnostic>,
    pub files: Vec<PathBuf>,
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

impl Project {
    pub fn for_file(file: &Path) -> Self {
        let file = canonical(file);
        let dir = file.parent().map(Path::to_path_buf).unwrap_or_default();

        let schema_path = dir
            .ancestors()
            .map(|dir| dir.join(SCHEMA_FILE_NAME))
            .find(|candidate| candidate.is_file());
        let loaded = schema_path.as_ref().and_then(|path| {
            SchemaFile::read(path)
                .ok()
                .map(|schema| (path.clone(), schema))
        });

        let Some((schema_path, mut schema)) = loaded else {
            return Self {
                stories: dir,
                single: None,
                schema: None,
                assets: None,
            };
        };

        let root = schema_path.parent().map(canonical).unwrap_or_default();
        let stories = canonical(&root.join(&schema.story_dir));
        if file.starts_with(&stories) {
            Self {
                stories,
                single: None,
                schema: Some(schema),
                assets: Some(root),
            }
        } else {
            schema.entry_scene = None;
            Self {
                stories: dir,
                single: Some(file),
                schema: Some(schema),
                assets: Some(root),
            }
        }
    }

    pub fn contains(&self, file: &Path) -> bool {
        let file = canonical(file);
        match &self.single {
            Some(single) => *single == file,
            None => file.starts_with(&self.stories),
        }
    }

    pub fn files(&self, overlays: &HashMap<PathBuf, String>) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = match &self.single {
            Some(single) => vec![single.clone()],
            None => story_files(&self.stories)
                .unwrap_or_default()
                .iter()
                .map(|path| canonical(path))
                .collect(),
        };
        for open in overlays.keys() {
            let is_story = open.extension().is_some_and(|ext| ext == "story");
            if is_story && self.contains(open) && !files.contains(open) {
                files.push(open.clone());
            }
        }
        files.sort();
        files
    }

    pub fn analyze(&self, overlays: &HashMap<PathBuf, String>) -> Analysis {
        let files = self.files(overlays);
        let sources: Vec<(String, String)> = files
            .iter()
            .filter_map(|path| {
                let text = match overlays.get(path) {
                    Some(text) => text.clone(),
                    None => fs::read_to_string(path).ok()?,
                };
                Some((path.display().to_string(), text))
            })
            .collect();

        let mut story = StoryVm::from_program(compile_sources(sources));
        let diagnostics = match &self.schema {
            Some(file) => story.prepare(file.schema.clone(), file.entry_scene.as_deref()),
            None => story.validate(),
        };
        Analysis {
            program: story.program().clone(),
            diagnostics,
            files,
        }
    }

    pub fn asset_ids(&self, dir: &str, extensions: &[&str]) -> Vec<String> {
        let Some(root) = &self.assets else {
            return Vec::new();
        };
        let mut ids: Vec<String> = fs::read_dir(root.join(dir))
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| extensions.contains(&ext.to_lowercase().as_str()))
            })
            .filter_map(|path| path.file_stem()?.to_str().map(str::to_string))
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }
}
