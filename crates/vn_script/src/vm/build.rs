use std::collections::HashMap;
use std::io;
use std::path::Path;

use super::StoryVm;
use crate::{
    Diagnostic, Program, Schema, compile_source, compile_sources, did_you_mean, read_sources,
    story_files,
};

impl StoryVm {
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let sources = read_sources(&[path.as_ref().to_path_buf()])?;
        Ok(Self::from_program(compile_sources(sources)))
    }

    pub fn from_dir(dir: impl AsRef<Path>) -> io::Result<Self> {
        let sources = read_sources(&story_files(dir)?)?;
        Ok(Self::from_program(compile_sources(sources)))
    }

    pub fn from_source(source: &str) -> Self {
        Self::from_program(compile_source(source))
    }

    pub fn from_program(program: Program) -> Self {
        let mut vm = Self {
            program,
            ip: 0,
            current_scene: None,
            variables: HashMap::new(),
            active_characters: HashMap::new(),
            positions: HashMap::new(),
            background: None,
            music: None,
            pending_transition: None,
            pending_choice: None,
            current: None,
            current_ip: None,
            schema: Schema::default(),
            entry: None,
            scene_events: false,
            entered: false,
            catalog: None,
        };
        vm.reset();
        vm
    }

    pub fn set_schema(&mut self, schema: Schema) {
        self.schema = schema;
        self.reset();
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn validate(&self) -> Vec<Diagnostic> {
        let mut diagnostics = self.schema.validate(&self.program);

        if let Some(entry) = &self.entry
            && !self.program.scenes.contains_key(entry)
        {
            diagnostics.insert(0, self.missing_entry(entry));
        }

        diagnostics
    }

    fn missing_entry(&self, scene: &str) -> Diagnostic {
        let scenes = self.program.scene_order.iter().map(String::as_str);
        Diagnostic::error(
            0,
            format!(
                "entry scene '{}' does not exist{}",
                scene,
                did_you_mean(scene, scenes)
            ),
        )
    }

    pub fn prepare(&mut self, schema: Schema, entry_scene: Option<&str>) -> Vec<Diagnostic> {
        self.set_schema(schema);

        let mut diagnostics = Vec::new();
        if let Some(scene) = entry_scene
            && self.set_entry_scene(scene).is_err()
        {
            diagnostics.push(self.missing_entry(scene));
        }

        diagnostics.extend(self.validate());
        diagnostics
    }
}
