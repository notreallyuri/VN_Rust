use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

pub const SHELF: &str = "shelf";
pub const REPORT: &str = "report";
pub const BOX: &str = "box";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Desk {
    room: String,
    examined: BTreeSet<String>,
    filed: BTreeMap<String, String>,
}

impl Desk {
    pub fn search(&mut self, room: &str) {
        self.room = room.to_string();
    }

    pub fn room(&self) -> &str {
        &self.room
    }

    pub fn examine(&mut self, spot: &str) -> bool {
        self.examined.insert(format!("{}:{}", self.room, spot))
    }

    pub fn examined(&self, spot: &str) -> bool {
        self.examined.contains(&format!("{}:{}", self.room, spot))
    }

    pub fn file(&mut self, item: &str, tray: &str) {
        self.filed.insert(item.to_string(), tray.to_string());
    }

    pub fn tray(&self, item: &str) -> &str {
        self.filed.get(item).map(String::as_str).unwrap_or(SHELF)
    }

    pub fn in_report(&self) -> impl Iterator<Item = &str> {
        self.filed
            .iter()
            .filter(|(_, tray)| tray.as_str() == REPORT)
            .map(|(item, _)| item.as_str())
    }
}

pub fn tray_name(tray: &str) -> &'static str {
    match tray {
        REPORT => "Report 222",
        BOX => "Back in Box 14",
        _ => "On the desk",
    }
}
