use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Evidence {
    items: BTreeMap<String, u32>,
}

impl Evidence {
    pub fn add(&mut self, item: &str, count: u32) {
        *self.items.entry(item.to_string()).or_insert(0) += count;
    }

    pub fn items(&self) -> impl Iterator<Item = (&str, u32)> {
        self.items
            .iter()
            .map(|(item, count)| (item.as_str(), *count))
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

pub fn describe(item: &str) -> (&'static str, &'static str) {
    match item {
        "box_14" => (
            "Box 14",
            "What was recovered from the Von Lucis residence. Never catalogued.",
        ),
        "field_report" => (
            "Field report",
            "The Santa Ilde field post, reports 203 to 221. The last one breaks off mid-word.",
        ),
        "verlaine_letter" => (
            "The Verlaine letter",
            "One word and a signature: \"Von Lucis. Three days.\"",
        ),
        "notebook_copy" => (
            "The administrator's notebook",
            "Francis's copy, made in one night. It includes the indentations of a torn page.",
        ),
        "wooden_horse" => (
            "A painted wooden horse",
            "It was not in the orphanage's inventory. He wanted it.",
        ),
        _ => ("Unknown item", "The Archive has no entry for this."),
    }
}
