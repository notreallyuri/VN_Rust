use std::collections::BTreeMap;

use crate::input::navigation::InputDevice;
use crate::input::pad::{PadFamily, PadLabels};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prompt {
    pub mouse: Option<String>,
    pub keyboard: String,
    pub gamepad: String,
}

impl Prompt {
    pub fn new(keyboard: impl Into<String>, gamepad: impl Into<String>) -> Self {
        Self {
            mouse: None,
            keyboard: keyboard.into(),
            gamepad: gamepad.into(),
        }
    }

    pub fn mouse(mut self, label: impl Into<String>) -> Self {
        self.mouse = Some(label.into());
        self
    }

    pub fn for_device(&self, device: InputDevice) -> &str {
        match device {
            InputDevice::Gamepad => &self.gamepad,
            InputDevice::Mouse => self.mouse.as_deref().unwrap_or(&self.keyboard),
            InputDevice::Keyboard => &self.keyboard,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Prompts {
    entries: BTreeMap<String, Prompt>,
    pub pad_labels: PadLabels,
}

impl Prompts {
    pub fn insert(&mut self, token: impl Into<String>, prompt: Prompt) {
        self.entries.insert(token.into(), prompt);
    }

    pub fn get(&self, token: &str) -> Option<&Prompt> {
        self.entries.get(token)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn tokens(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }

    pub fn fill(&self, template: &str, device: InputDevice, pad: PadFamily) -> String {
        let fields: Vec<(&str, String)> = self
            .entries
            .iter()
            .map(|(token, prompt)| {
                let label = prompt.for_device(device);
                let label = match device {
                    InputDevice::Gamepad => self.pad_labels.translate(label, pad),
                    _ => label.to_string(),
                };
                (token.as_str(), label)
            })
            .collect();
        let fields: Vec<(&str, &str)> = fields
            .iter()
            .map(|(token, label)| (*token, label.as_str()))
            .collect();
        crate::ui::labels::fill(template, &fields)
    }
}
