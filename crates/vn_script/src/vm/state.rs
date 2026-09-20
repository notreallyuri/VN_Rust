use std::collections::HashMap;

use super::{Event, StoryVm, VmError};
use crate::{Catalog, Instruction, Position, Program, Value, VarType};

impl StoryVm {
    pub fn set_entry_scene(&mut self, scene_id: impl Into<String>) -> Result<(), VmError> {
        let scene_id = scene_id.into();
        if !self.program.scenes.contains_key(&scene_id) {
            return Err(VmError::UnknownScene(scene_id));
        }

        self.entry = Some(scene_id);
        self.reset();
        Ok(())
    }

    pub fn entry_scene(&self) -> Option<&str> {
        self.entry.as_deref().or_else(|| self.program.entry_scene())
    }

    pub fn reset(&mut self) {
        self.variables = self.schema.defaults().into_iter().collect();
        self.active_characters.clear();
        self.positions.clear();
        self.background = None;
        self.music = None;
        self.pending_transition = None;
        self.pending_choice = None;
        self.current = None;
        self.entered = false;
        self.reset_position();
    }

    pub fn set_catalog(&mut self, catalog: Option<Catalog>) {
        self.catalog = catalog;
    }

    pub fn catalog(&self) -> Option<&Catalog> {
        self.catalog.as_ref()
    }

    pub fn language(&self) -> Option<&str> {
        self.catalog
            .as_ref()
            .map(|catalog| catalog.language.as_str())
    }

    pub(super) fn localized<'a>(&'a self, ip: usize, text: &'a str) -> &'a str {
        let Some(catalog) = &self.catalog else {
            return text;
        };
        let Some(file) = self.program.file(ip) else {
            return text;
        };
        catalog.text(file, text).unwrap_or(text)
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn current_scene(&self) -> Option<&str> {
        self.current_scene.as_deref()
    }

    pub fn active_characters(&self) -> &HashMap<String, String> {
        &self.active_characters
    }

    pub fn position(&self, character: &str) -> Option<Position> {
        self.positions.get(character).copied()
    }

    pub fn background(&self) -> Option<&str> {
        self.background.as_deref()
    }

    pub fn line_key(&self) -> Option<u64> {
        if !matches!(self.current, Some(Event::Say { .. })) {
            return None;
        }
        let scene = self.current_scene.as_deref()?;
        let Some(Instruction::Say { char_id, text }) =
            self.program.instructions.get(self.ip.checked_sub(1)?)
        else {
            return None;
        };
        let key = format!("{}\0{}\0{}", scene, char_id.as_deref().unwrap_or(""), text);
        Some(crate::types::instructions::fnv1a(key.as_bytes()))
    }

    pub fn music(&self) -> Option<&str> {
        self.music.as_deref()
    }

    pub fn variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }

    pub fn variable(&self, id: &str) -> Option<&Value> {
        self.variables.get(id)
    }

    pub fn set_variable(&mut self, id: impl Into<String>, value: Value) -> Result<(), VmError> {
        let id = id.into();

        if !self.schema.variables.is_empty() {
            let def = self
                .schema
                .variables
                .get(&id)
                .ok_or_else(|| VmError::UnknownVariable(id.clone()))?;

            def.ty
                .check(&value)
                .map_err(|message| VmError::TypeMismatch {
                    variable: id.clone(),
                    message,
                })?;
        }

        self.variables.insert(id, value);
        Ok(())
    }

    pub fn variable_type(&self, id: &str) -> Option<&VarType> {
        self.schema.variables.get(id).map(|def| &def.ty)
    }
}
