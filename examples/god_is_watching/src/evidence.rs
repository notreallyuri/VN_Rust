use std::collections::BTreeMap;

use novn::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Evidence {
    items: BTreeMap<EvidenceItem, u32>,
}

#[derive(StoryWord, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceItem {
    Box14,
    FieldReport,
    VerlaineLetter,
    NotebookCopy,
    WoodenHorse,
}

impl EvidenceItem {
    pub fn describe(self) -> (&'static str, &'static str) {
        match self {
            EvidenceItem::Box14 => (
                "Box 14",
                "What was recovered from the Von Lucis residence. Never catalogued.",
            ),
            EvidenceItem::FieldReport => (
                "Field Report",
                "The Santa Ilde field post, reports 203 to 221. The last one breaks off mid-word.",
            ),
            EvidenceItem::VerlaineLetter => (
                "Verlaine Letter",
                "One word and a signature: \"Von Lucis. Three days.\"",
            ),
            EvidenceItem::NotebookCopy => (
                "The administrator's notebook",
                "Francis's copy, made in one night. It includes the indentations of a torn page.",
            ),
            EvidenceItem::WoodenHorse => (
                "A painted wooden horse",
                "It was not in the orphanage's inventory. He wanted it.",
            ),
        }
    }
}

impl Evidence {
    pub fn add(&mut self, item: EvidenceItem, count: u32) {
        *self.items.entry(item).or_insert(0) += count;
    }

    pub fn items(&self) -> impl Iterator<Item = (EvidenceItem, u32)> {
        self.items.iter().map(|(item, count)| (*item, *count))
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
