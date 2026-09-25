use std::any::{Any, type_name};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value as Json;

type Initial = Box<dyn Fn() -> Box<dyn Any>>;
type Save = Box<dyn Fn(&dyn Any) -> serde_json::Result<Json>>;
type Load = Box<dyn Fn(Json) -> serde_json::Result<Box<dyn Any>>>;

struct Entry {
    type_name: &'static str,
    value: Box<dyn Any>,
    initial: Initial,
    save: Save,
    load: Load,
}

#[derive(Debug)]
pub struct StateError {
    pub key: String,
    pub source: serde_json::Error,
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "state '{}': {}", self.key, self.source)
    }
}

impl std::error::Error for StateError {}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct StateLoadReport {
    pub missing: Vec<String>,
    pub unknown: Vec<String>,
}

pub struct PendingState {
    values: Vec<(String, Box<dyn Any>)>,
    pub report: StateLoadReport,
}

#[derive(Default)]
pub struct GameState {
    entries: HashMap<String, Entry>,
}

pub fn state_key<T>() -> &'static str {
    let full = type_name::<T>();
    let base = full.split('<').next().unwrap_or(full);
    base.rsplit("::").next().unwrap_or(base)
}

impl GameState {
    pub fn insert<T>(&mut self, value: T)
    where
        T: Clone + Serialize + DeserializeOwned + 'static,
    {
        let key = state_key::<T>().to_string();

        if let Some(existing) = self.entries.get(&key)
            && existing.type_name != type_name::<T>()
        {
            panic!(
                "state key '{}' is used by both {} and {}; rename one of the types",
                key,
                existing.type_name,
                type_name::<T>()
            );
        }

        let initial = value.clone();
        self.entries.insert(
            key,
            Entry {
                type_name: type_name::<T>(),
                value: Box::new(value),
                initial: Box::new(move || Box::new(initial.clone())),
                save: Box::new(|value| {
                    serde_json::to_value(value.downcast_ref::<T>().expect("state type matches"))
                }),
                load: Box::new(|json| Ok(Box::new(serde_json::from_value::<T>(json)?))),
            },
        );
    }

    pub fn try_get<T: 'static>(&self) -> Option<&T> {
        self.entries.get(state_key::<T>())?.value.downcast_ref()
    }

    pub fn try_get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.entries.get_mut(state_key::<T>())?.value.downcast_mut()
    }

    pub fn get<T: 'static>(&self) -> &T {
        self.try_get()
            .unwrap_or_else(|| panic!("{} is not registered with VnApp::state", type_name::<T>()))
    }

    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        self.try_get_mut()
            .unwrap_or_else(|| panic!("{} is not registered with VnApp::state", type_name::<T>()))
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    pub fn reset(&mut self) {
        for entry in self.entries.values_mut() {
            entry.value = (entry.initial)();
        }
    }

    pub fn to_json(&self) -> Result<BTreeMap<String, Json>, StateError> {
        self.entries
            .iter()
            .map(|(key, entry)| {
                (entry.save)(entry.value.as_ref())
                    .map(|json| (key.clone(), json))
                    .map_err(|source| StateError {
                        key: key.clone(),
                        source,
                    })
            })
            .collect()
    }

    pub fn prepare_load(&self, saved: &BTreeMap<String, Json>) -> Result<PendingState, StateError> {
        let mut values = Vec::new();
        let mut report = StateLoadReport::default();

        for (key, entry) in &self.entries {
            match saved.get(key) {
                Some(json) => {
                    let value = (entry.load)(json.clone()).map_err(|source| StateError {
                        key: key.clone(),
                        source,
                    })?;
                    values.push((key.clone(), value));
                }
                None => {
                    values.push((key.clone(), (entry.initial)()));
                    report.missing.push(key.clone());
                }
            }
        }

        report.unknown = saved
            .keys()
            .filter(|key| !self.entries.contains_key(*key))
            .cloned()
            .collect();
        report.missing.sort();

        Ok(PendingState { values, report })
    }

    pub(crate) fn restore(&mut self, saved: &BTreeMap<String, Json>) -> Vec<StateError> {
        let mut failed = Vec::new();
        for (key, entry) in &mut self.entries {
            if let Some(json) = saved.get(key) {
                match (entry.load)(json.clone()) {
                    Ok(value) => entry.value = value,
                    Err(source) => failed.push(StateError {
                        key: key.clone(),
                        source,
                    }),
                }
            }
        }
        failed.sort_by(|a, b| a.key.cmp(&b.key));
        failed
    }

    pub fn apply(&mut self, pending: PendingState) -> StateLoadReport {
        for (key, value) in pending.values {
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.value = value;
            }
        }
        pending.report
    }
}
