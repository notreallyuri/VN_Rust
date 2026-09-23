use serde::{Deserialize, Serialize};

use crate::{Position, Transition};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Say {
        speaker: Option<String>,
        text: String,
    },
    Choice {
        options: Vec<ChoiceOption>,
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceOption {
    pub text: String,
    pub index: usize,
    #[serde(default = "yes", skip_serializing_if = "is_yes")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

impl ChoiceOption {
    pub fn new(text: impl Into<String>, index: usize) -> Self {
        Self {
            text: text.into(),
            index,
            enabled: true,
            reason: None,
            image: None,
            preview: None,
        }
    }
}

impl std::fmt::Debug for ChoiceOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}#{}", self.text, self.index)?;
        if !self.enabled {
            match &self.reason {
                Some(reason) => write!(f, " unavailable: {:?}", reason)?,
                None => write!(f, " unavailable")?,
            }
        }
        if let Some(image) = &self.image {
            write!(f, " image: {}", image)?;
        }
        if let Some(preview) = &self.preview {
            write!(f, " preview: {}", preview)?;
        }
        Ok(())
    }
}

fn yes() -> bool {
    true
}

fn is_yes(value: &bool) -> bool {
    *value
}

impl Event {
    pub fn is_blocking(&self) -> bool {
        matches!(self, Event::Say { .. } | Event::Choice { .. } | Event::End)
    }
}
