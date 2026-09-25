use std::collections::BTreeSet;

use novn::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(StoryWord, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ending {
    Report,
    Silence,
    Keeper,
}

impl Ending {
    pub fn name(self) -> &'static str {
        match self {
            Ending::Report => "The report",
            Ending::Silence => "The silence",
            Ending::Keeper => "The keeper",
        }
    }
}

#[derive(StoryWord, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Note {
    DebtPaid,
    #[word("folio_41")]
    Folio41,
    HotWater,
}

impl Note {
    pub fn text(self) -> &'static str {
        match self {
            Note::DebtPaid => {
                "A Von Lucis debt was marked paid three days after the Verlaine letter."
            }
            Note::Folio41 => {
                "The torn page described folio 41 of a register, folded into the boy's cloths."
            }
            Note::HotWater => "\"Your hot water, miss.\" Sister Clara is Adelaide Roque.",
        }
    }
}

#[derive(StoryWord, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Achievement {
    ThoroughReader,
    EndingReport,
    EndingSilence,
    EndingKeeper,
}

impl Achievement {
    pub fn name(self) -> &'static str {
        match self {
            Achievement::ThoroughReader => {
                "Thorough reader: opened the letter and read the torn page"
            }
            Achievement::EndingReport => "Ending: the report",
            Achievement::EndingSilence => "Ending: the silence",
            Achievement::EndingKeeper => "Ending: the keeper",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Journal {
    notes: Vec<Note>,
    decisions: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CaseLedger {
    endings: BTreeSet<Ending>,
}

impl CaseLedger {
    pub fn close(&mut self, ending: Ending) {
        self.endings.insert(ending);
    }

    pub fn has(&self, ending: Ending) -> bool {
        self.endings.contains(&ending)
    }

    pub fn closed(&self) -> usize {
        self.endings.len()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Achievements {
    unlocked: BTreeSet<Achievement>,
}

impl Achievements {
    pub fn unlock(&mut self, key: Achievement) -> bool {
        self.unlocked.insert(key)
    }

    pub fn unlocked(&self) -> impl Iterator<Item = Achievement> {
        self.unlocked.iter().copied()
    }
}

impl Journal {
    pub fn note(&mut self, key: Note) -> bool {
        if self.notes.contains(&key) {
            return false;
        }
        self.notes.push(key);
        true
    }

    pub fn decide(&mut self, text: &str) {
        self.decisions.push(text.to_string());
    }

    pub fn notes(&self) -> impl Iterator<Item = Note> {
        self.notes.iter().copied()
    }

    pub fn recent_decisions(&self, count: usize) -> impl Iterator<Item = &str> {
        let skip = self.decisions.len().saturating_sub(count);
        self.decisions.iter().skip(skip).map(String::as_str)
    }
}

pub fn chapter_title(scene: &str) -> Option<&'static str> {
    Some(match scene {
        "archive_start" => "Prologue: The Archive",
        "box_start" => "Chapter I: Box 14",
        "notebook_start" => "Chapter II: The Notebook",
        "reports_start" => "Chapter III: Field Reports",
        "ilde_start" => "Chapter IV: Santa Ilde, 1903",
        "report_start" => "Chapter V: The Report",
        _ => return None,
    })
}
