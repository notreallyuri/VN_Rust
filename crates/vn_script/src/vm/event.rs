use serde::{Deserialize, Serialize};

use crate::{Position, Transition};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Say {
        speaker: Option<String>,
        text: String,
    },
    Choice {
        options: Vec<String>,
    },
    Show {
        character: String,
        image: String,
        position: Option<Position>,
        transition: Option<Transition>,
    },
    Background {
        image: Option<String>,
        transition: Option<Transition>,
    },
    Music {
        track: Option<String>,
    },
    Sound {
        id: String,
    },
    Voice {
        id: String,
    },
    Hide {
        character: String,
        transition: Option<Transition>,
    },
    Clear {
        transition: Option<Transition>,
    },
    Call {
        command: String,
        args: Vec<String>,
    },
    Commit,
    SceneEnter {
        scene: String,
    },
    End,
}

impl Event {
    pub fn is_blocking(&self) -> bool {
        matches!(self, Event::Say { .. } | Event::Choice { .. } | Event::End)
    }
}
