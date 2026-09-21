use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PadFamily {
    #[default]
    Xbox,
    PlayStation,
    Nintendo,
}

impl PadFamily {
    pub fn detect(name: &str) -> Self {
        let name = name.to_lowercase();
        let any = |words: &[&str]| words.iter().any(|word| name.contains(word));
        if any(&[
            "dualsense",
            "dualshock",
            "playstation",
            "ps5",
            "ps4",
            "ps3",
            "sony",
        ]) {
            PadFamily::PlayStation
        } else if any(&["nintendo", "switch", "joy-con", "joycon"]) {
            PadFamily::Nintendo
        } else {
            PadFamily::Xbox
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PadButton {
    FaceDown,
    FaceRight,
    FaceLeft,
    FaceUp,
    Start,
    Select,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    LeftStick,
    RightStick,
}

impl PadButton {
    pub const ALL: [PadButton; 12] = [
        PadButton::FaceDown,
        PadButton::FaceRight,
        PadButton::FaceLeft,
        PadButton::FaceUp,
        PadButton::Start,
        PadButton::Select,
        PadButton::LeftBumper,
        PadButton::RightBumper,
        PadButton::LeftTrigger,
        PadButton::RightTrigger,
        PadButton::LeftStick,
        PadButton::RightStick,
    ];

    pub fn from_xbox(word: &str) -> Option<Self> {
        PadButton::ALL
            .into_iter()
            .find(|button| button.label(PadFamily::Xbox) == word)
    }

    pub fn label(self, family: PadFamily) -> &'static str {
        use PadButton::*;
        match family {
            PadFamily::Xbox => match self {
                FaceDown => "A",
                FaceRight => "B",
                FaceLeft => "X",
                FaceUp => "Y",
                Start => "Start",
                Select => "Select",
                LeftBumper => "LB",
                RightBumper => "RB",
                LeftTrigger => "LT",
                RightTrigger => "RT",
                LeftStick => "LS",
                RightStick => "RS",
            },
            PadFamily::PlayStation => match self {
                FaceDown => "Cross",
                FaceRight => "Circle",
                FaceLeft => "Square",
                FaceUp => "Triangle",
                Start => "Options",
                Select => "Create",
                LeftBumper => "L1",
                RightBumper => "R1",
                LeftTrigger => "L2",
                RightTrigger => "R2",
                LeftStick => "L3",
                RightStick => "R3",
            },
            PadFamily::Nintendo => match self {
                FaceDown => "B",
                FaceRight => "A",
                FaceLeft => "Y",
                FaceUp => "X",
                Start => "+",
                Select => "-",
                LeftBumper => "L",
                RightBumper => "R",
                LeftTrigger => "ZL",
                RightTrigger => "ZR",
                LeftStick => "LS",
                RightStick => "RS",
            },
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PadLabels {
    overrides: BTreeMap<(PadFamily, PadButton), String>,
}

impl PadLabels {
    pub fn set(&mut self, family: PadFamily, button: PadButton, label: impl Into<String>) {
        self.overrides.insert((family, button), label.into());
    }

    pub fn label(&self, button: PadButton, family: PadFamily) -> &str {
        self.overrides
            .get(&(family, button))
            .map(String::as_str)
            .unwrap_or_else(|| button.label(family))
    }

    pub fn overrides(&self) -> impl Iterator<Item = &str> {
        self.overrides.values().map(String::as_str)
    }

    pub fn translate(&self, text: &str, family: PadFamily) -> String {
        let mut out = String::with_capacity(text.len());
        let mut word = String::new();
        let flush = |word: &mut String, out: &mut String| {
            match PadButton::from_xbox(word) {
                Some(button) => out.push_str(self.label(button, family)),
                None => out.push_str(word),
            }
            word.clear();
        };
        for c in text.chars() {
            if c.is_alphanumeric() {
                word.push(c);
            } else {
                flush(&mut word, &mut out);
                out.push(c);
            }
        }
        flush(&mut word, &mut out);
        out
    }
}
