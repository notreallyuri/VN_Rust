use std::fmt;
use std::rc::Rc;

use serde_json::Value as Json;

pub type MigrationFn = dyn Fn(&mut SaveMigration) -> Result<(), String>;

#[derive(Clone, Default)]
pub struct Migrations {
    steps: Vec<(u32, Rc<MigrationFn>)>,
}

impl fmt::Debug for Migrations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.steps.iter().map(|(from, _)| from))
            .finish()
    }
}

impl Migrations {
    pub fn add(
        &mut self,
        from: u32,
        migration: impl Fn(&mut SaveMigration) -> Result<(), String> + 'static,
    ) {
        self.steps.push((from, Rc::new(migration)));
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub(super) fn run(&self, json: &mut Json, from: u32, to: u32) -> Result<(), (u32, String)> {
        for version in from..to {
            for (_, migration) in self.steps.iter().filter(|(step, _)| *step == version) {
                migration(&mut SaveMigration { json }).map_err(|message| (version, message))?;
            }
        }
        Ok(())
    }
}

pub struct SaveMigration<'a> {
    json: &'a mut Json,
}

impl SaveMigration<'_> {
    pub fn json(&mut self) -> &mut Json {
        self.json
    }

    fn states(&mut self) -> Vec<&mut serde_json::Map<String, Json>> {
        let Json::Object(file) = &mut *self.json else {
            return Vec::new();
        };
        let mut maps = Vec::new();
        for (key, value) in file.iter_mut() {
            match (key.as_str(), value) {
                ("state", Json::Object(state)) => maps.push(state),
                ("rollback", Json::Array(checkpoints)) => maps.extend(
                    checkpoints
                        .iter_mut()
                        .filter_map(|c| match c.get_mut("state") {
                            Some(Json::Object(state)) => Some(state),
                            _ => None,
                        }),
                ),
                _ => {}
            }
        }
        maps
    }

    fn variables(&mut self) -> Vec<&mut serde_json::Map<String, Json>> {
        let Json::Object(file) = &mut *self.json else {
            return Vec::new();
        };
        let mut stories = Vec::new();
        for (key, value) in file.iter_mut() {
            match (key.as_str(), value) {
                ("story", story) => stories.push(story),
                ("rollback", Json::Array(checkpoints)) => {
                    stories.extend(checkpoints.iter_mut().filter_map(|c| c.get_mut("story")))
                }
                _ => {}
            }
        }
        stories
            .into_iter()
            .filter_map(|story| match story.get_mut("variables") {
                Some(Json::Object(variables)) => Some(variables),
                _ => None,
            })
            .collect()
    }

    pub fn state(
        &mut self,
        key: &str,
        mut edit: impl FnMut(&mut Json) -> Result<(), String>,
    ) -> Result<(), String> {
        for state in self.states() {
            if let Some(value) = state.get_mut(key) {
                edit(value)?;
            }
        }
        Ok(())
    }

    pub fn rename_state(&mut self, from: &str, to: &str) {
        for state in self.states() {
            if let Some(value) = state.remove(from) {
                state.insert(to.to_string(), value);
            }
        }
    }

    pub fn remove_state(&mut self, key: &str) {
        for state in self.states() {
            state.remove(key);
        }
    }

    pub fn variable(
        &mut self,
        name: &str,
        mut edit: impl FnMut(&mut Json) -> Result<(), String>,
    ) -> Result<(), String> {
        for variables in self.variables() {
            if let Some(value) = variables.get_mut(name) {
                edit(value)?;
            }
        }
        Ok(())
    }

    pub fn rename_variable(&mut self, from: &str, to: &str) {
        for variables in self.variables() {
            if let Some(value) = variables.remove(from) {
                variables.insert(to.to_string(), value);
            }
        }
    }

    pub fn remove_variable(&mut self, name: &str) {
        for variables in self.variables() {
            variables.remove(name);
        }
    }
}

pub(super) fn format_migrations() -> Migrations {
    let mut migrations = Migrations::default();
    migrations.add(1, |migration| {
        migration.spoken_log();
        Ok(())
    });
    migrations
}

impl SaveMigration<'_> {
    fn spoken_log(&mut self) {
        let Some(Json::Array(entries)) = self.json.get_mut("log") else {
            return;
        };
        for entry in entries.iter_mut() {
            let Json::Object(entry) = entry else {
                continue;
            };
            for kind in ["line", "choice"] {
                if let Some(Json::Object(fields)) = entry.get_mut(kind)
                    && !fields.contains_key("source")
                    && let Some(text) = fields.remove("text")
                {
                    fields.insert("source".into(), text);
                }
            }
        }
    }
}
