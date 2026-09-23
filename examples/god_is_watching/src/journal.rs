use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Journal {
    notes: Vec<String>,
    decisions: Vec<String>,
}

pub const ENDINGS: [&str; 3] = ["report", "silence", "keeper"];

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CaseLedger {
    endings: BTreeSet<String>,
}

impl CaseLedger {
    pub fn close(&mut self, ending: &str) {
        self.endings.insert(ending.to_string());
    }

    pub fn has(&self, ending: &str) -> bool {
        self.endings.contains(ending)
    }

    pub fn closed(&self) -> usize {
        self.endings.len()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Achievements {
    unlocked: BTreeSet<String>,
}

impl Achievements {
    pub fn unlock(&mut self, key: &str) -> bool {
        self.unlocked.insert(key.to_string())
    }

    pub fn unlocked(&self) -> impl Iterator<Item = &str> {
        self.unlocked.iter().map(String::as_str)
    }
}

impl Journal {
    pub fn note(&mut self, key: &str) -> bool {
        if self.notes.iter().any(|note| note == key) {
            return false;
        }
        self.notes.push(key.to_string());
        true
    }

    pub fn decide(&mut self, text: &str) {
        self.decisions.push(text.to_string());
    }

    pub fn notes(&self) -> impl Iterator<Item = &str> {
        self.notes.iter().map(String::as_str)
    }

    pub fn recent_decisions(&self, count: usize) -> impl Iterator<Item = &str> {
        let skip = self.decisions.len().saturating_sub(count);
        self.decisions.iter().skip(skip).map(String::as_str)
    }
}

pub fn note_text(key: &str) -> &'static str {
    match key {
        "debt_paid" => "A Von Lucis debt was marked paid three days after the Verlaine letter.",
        "folio_41" => {
            "The torn page described folio 41 of a register, folded into the boy's cloths."
        }
        "hot_water" => "\"Your hot water, miss.\" Sister Clara is Adelaide Roque.",
        _ => "An unreadable note.",
    }
}

pub fn ending_name(key: &str) -> &'static str {
    match key {
        "report" => "The report",
        "silence" => "The silence",
        "keeper" => "The keeper",
        _ => "An unknown ending",
    }
}

pub fn achievement_name(key: &str) -> &'static str {
    match key {
        "thorough_reader" => "Thorough reader: opened the letter and read the torn page",
        "ending_report" => "Ending: the report",
        "ending_silence" => "Ending: the silence",
        "ending_keeper" => "Ending: the keeper",
        _ => "Unknown achievement",
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
