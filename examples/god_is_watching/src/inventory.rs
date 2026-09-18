use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Inventory {
    items: BTreeMap<String, u32>,
}

impl Inventory {
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

pub fn display_name(id: &str) -> String {
    let words = id.split('_').filter(|w| !w.is_empty());
    let mut name = words.collect::<Vec<_>>().join(" ");
    if let Some(first) = name.get(..1) {
        name.replace_range(..1, &first.to_uppercase());
    }
    name
}
