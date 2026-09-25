use std::collections::{BTreeMap, BTreeSet};

use novn::prelude::*;
use serde::{Deserialize, Serialize};

use crate::evidence::EvidenceItem;

#[derive(StoryWord, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Room {
    Study,
}

#[derive(StoryWord, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tray {
    Shelf,
    Report,
    Box,
}

impl Tray {
    pub fn name(self) -> &'static str {
        match self {
            Tray::Shelf => "On the desk",
            Tray::Report => "Report 222",
            Tray::Box => "Back in Box 14",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Desk {
    room: Option<Room>,
    examined: BTreeSet<String>,
    filed: BTreeMap<EvidenceItem, Tray>,
}

impl Desk {
    pub fn search(&mut self, room: Room) {
        self.room = Some(room);
    }

    pub fn room(&self) -> Option<Room> {
        self.room
    }

    pub fn examine(&mut self, spot: &str) -> bool {
        self.examined.insert(self.spot(spot))
    }

    pub fn examined(&self, spot: &str) -> bool {
        self.examined.contains(&self.spot(spot))
    }

    fn spot(&self, spot: &str) -> String {
        match self.room {
            Some(room) => format!("{}:{}", room, spot),
            None => spot.to_string(),
        }
    }

    pub fn file(&mut self, item: EvidenceItem, tray: Tray) {
        self.filed.insert(item, tray);
    }

    pub fn tray(&self, item: EvidenceItem) -> Tray {
        self.filed.get(&item).copied().unwrap_or(Tray::Shelf)
    }

    pub fn in_report(&self) -> impl Iterator<Item = EvidenceItem> {
        self.filed
            .iter()
            .filter(|(_, tray)| **tray == Tray::Report)
            .map(|(item, _)| *item)
    }
}
