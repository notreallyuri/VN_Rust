use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    Int(i32),
    Enum(String),
    String(String),
}

impl Value {
    pub fn literal(&self) -> String {
        match self {
            Value::String(text) => format!("\"{}\"", text),
            other => other.to_string(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(n) => write!(f, "{}", n),
            Value::Enum(member) => write!(f, "{}", member),
            Value::String(text) => write!(f, "{}", text),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    FarLeft,
    Left,
    Center,
    Right,
    FarRight,
}

impl Position {
    pub const ALL: [Position; 5] = [
        Position::FarLeft,
        Position::Left,
        Position::Center,
        Position::Right,
        Position::FarRight,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Position::FarLeft => "far_left",
            Position::Left => "left",
            Position::Center => "center",
            Position::Right => "right",
            Position::FarRight => "far_right",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneMode {
    #[default]
    Adv,
    Nvl,
}

impl SceneMode {
    pub const ALL: [SceneMode; 2] = [SceneMode::Adv, SceneMode::Nvl];

    pub fn name(self) -> &'static str {
        match self {
            SceneMode::Adv => "adv",
            SceneMode::Nvl => "nvl",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.name() == name)
    }

    pub fn is_nvl(self) -> bool {
        self == SceneMode::Nvl
    }
}

impl std::fmt::Display for SceneMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionKind {
    Dissolve,
    Fade,
    SlideLeft,
    SlideRight,
    Shake,
    Flash,
}

impl TransitionKind {
    pub const ALL: [TransitionKind; 6] = [
        TransitionKind::Dissolve,
        TransitionKind::Fade,
        TransitionKind::SlideLeft,
        TransitionKind::SlideRight,
        TransitionKind::Shake,
        TransitionKind::Flash,
    ];

    pub fn name(self) -> &'static str {
        match self {
            TransitionKind::Dissolve => "dissolve",
            TransitionKind::Fade => "fade",
            TransitionKind::SlideLeft => "slide_left",
            TransitionKind::SlideRight => "slide_right",
            TransitionKind::Shake => "shake",
            TransitionKind::Flash => "flash",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
    }

    pub fn default_millis(self) -> u32 {
        match self {
            TransitionKind::Dissolve => 500,
            TransitionKind::Fade => 1000,
            TransitionKind::SlideLeft | TransitionKind::SlideRight => 600,
            TransitionKind::Shake => 400,
            TransitionKind::Flash => 300,
        }
    }

    pub fn is_effect(self) -> bool {
        matches!(self, TransitionKind::Shake | TransitionKind::Flash)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Transition {
    pub kind: TransitionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub millis: Option<u32>,
}

impl Transition {
    pub fn new(kind: TransitionKind) -> Self {
        Self { kind, millis: None }
    }

    pub fn seconds(&self) -> f32 {
        self.millis.unwrap_or_else(|| self.kind.default_millis()) as f32 / 1000.0
    }
}

impl std::fmt::Display for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.millis {
            Some(millis) => write!(f, "{} {}", self.kind.name(), millis as f32 / 1000.0),
            None => write!(f, "{}", self.kind.name()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Comparison {
    Equal,
    NotEqual,
    Gte,
    Lte,
    Gt,
    Lt,
}

impl Comparison {
    pub fn symbol(self) -> &'static str {
        match self {
            Comparison::Equal => "==",
            Comparison::NotEqual => "!=",
            Comparison::Gte => ">=",
            Comparison::Lte => "<=",
            Comparison::Gt => ">",
            Comparison::Lt => "<",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    Test {
        var_id: String,
        op: Comparison,
        value: Value,
    },
    All(Vec<Condition>),
    Any(Vec<Condition>),
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let join = |f: &mut std::fmt::Formatter<'_>, parts: &[Condition], sep: &str| {
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    write!(f, " {} ", sep)?;
                }
                write!(f, "{}", part)?;
            }
            Ok(())
        };

        match self {
            Condition::Test { var_id, op, value } => {
                write!(f, "{} {} {}", var_id, op.symbol(), value.literal())
            }
            Condition::All(parts) => join(f, parts, "&&"),
            Condition::Any(parts) => join(f, parts, "||"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionGate {
    pub condition: Condition,
    #[serde(default, skip_serializing_if = "is_false")]
    pub negated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl OptionGate {
    pub fn keyword(&self) -> &'static str {
        match self.negated {
            true => "unless",
            false => "when",
        }
    }

    pub fn passes(&self, met: bool) -> bool {
        met != self.negated
    }

    pub fn hides(&self) -> bool {
        self.reason.is_none()
    }
}

impl std::fmt::Display for OptionGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.keyword(), self.condition)?;
        match &self.reason {
            Some(reason) => write!(f, " \"{}\"", reason),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceArm {
    pub text: String,
    pub target: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<OptionGate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

impl ChoiceArm {
    pub fn new(text: impl Into<String>, target: usize) -> Self {
        Self {
            text: text.into(),
            target,
            gate: None,
            image: None,
            preview: None,
        }
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    Show {
        char_id: String,
        img_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        position: Option<Position>,
    },
    Background {
        image: Option<String>,
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
        char_id: String,
    },
    Clear,
    Say {
        char_id: Option<String>,
        text: String,
    },
    Jump {
        scene_id: String,
    },
    JumpIfFalse {
        condition: Condition,
        jump_to_index: usize,
    },
    Choice {
        options: Vec<ChoiceArm>,
        after: usize,
    },
    Set {
        var_id: String,
        value: Value,
    },
    Add {
        var_id: String,
        amount: i32,
    },
    Call {
        command: String,
        args: Vec<String>,
    },
    Pause,
    Commit,
    With(Transition),
    Goto(usize),
    End,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub file: usize,
    pub line: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    #[serde(default)]
    pub locations: Vec<Location>,
    #[serde(default)]
    pub files: Vec<String>,
    pub scenes: HashMap<String, usize>,
    pub scene_order: Vec<String>,
    #[serde(default)]
    pub scene_locations: HashMap<String, Location>,
    #[serde(default)]
    pub scene_modes: HashMap<String, SceneMode>,
    #[serde(skip)]
    pub diagnostics: Vec<crate::Diagnostic>,
}

impl Program {
    pub fn location(&self, index: usize) -> Location {
        self.locations.get(index).copied().unwrap_or_default()
    }

    pub fn line(&self, index: usize) -> usize {
        self.location(index).line
    }

    pub fn file(&self, index: usize) -> Option<&str> {
        self.file_name(self.location(index).file)
    }

    pub fn file_name(&self, file: usize) -> Option<&str> {
        self.files.get(file).map(String::as_str)
    }

    pub fn sort_diagnostics(&self, diagnostics: &mut [crate::Diagnostic]) {
        let file_order = |d: &crate::Diagnostic| {
            d.file
                .as_deref()
                .and_then(|file| self.files.iter().position(|f| f == file))
        };
        diagnostics.sort_by_key(|d| (file_order(d), d.line));
    }

    pub fn scene_mode(&self, scene_id: &str) -> SceneMode {
        self.scene_modes.get(scene_id).copied().unwrap_or_default()
    }

    pub fn entry_scene(&self) -> Option<&str> {
        self.scene_order.first().map(String::as_str)
    }

    pub fn scene_range(&self, scene_id: &str) -> Option<std::ops::Range<usize>> {
        let start = *self.scenes.get(scene_id)?;
        let end = self
            .scenes
            .values()
            .copied()
            .filter(|&other| other > start)
            .min()
            .unwrap_or(self.instructions.len());
        Some(start..end)
    }

    pub fn scene_fingerprint(&self, scene_id: &str) -> Option<u64> {
        let range = self.scene_range(scene_id)?;
        let base = range.start;

        let relative: Vec<Instruction> = self.instructions[range]
            .iter()
            .map(|instr| instr.relative_to(base))
            .collect();
        let canonical = serde_json::to_string(&relative).expect("instructions serialize");

        Some(fnv1a(canonical.as_bytes()))
    }

    pub fn unknown_jump_targets(&self) -> Vec<&str> {
        self.instructions
            .iter()
            .filter_map(|instr| match instr {
                Instruction::Jump { scene_id } if !self.scenes.contains_key(scene_id) => {
                    Some(scene_id.as_str())
                }
                _ => None,
            })
            .collect()
    }
}

impl Instruction {
    fn relative_to(&self, base: usize) -> Instruction {
        let shift = |target: usize| target.saturating_sub(base);

        match self {
            Instruction::Goto(target) => Instruction::Goto(shift(*target)),
            Instruction::JumpIfFalse {
                condition,
                jump_to_index,
            } => Instruction::JumpIfFalse {
                condition: condition.clone(),
                jump_to_index: shift(*jump_to_index),
            },
            Instruction::Choice { options, after } => Instruction::Choice {
                options: options
                    .iter()
                    .map(|arm| ChoiceArm {
                        target: shift(arm.target),
                        ..arm.clone()
                    })
                    .collect(),
                after: shift(*after),
            },
            other => other.clone(),
        }
    }
}

pub(crate) fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, &byte| {
        (hash ^ byte as u64).wrapping_mul(0x100000001b3)
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneInstruction {
    pub id: String,
    pub instructions: Vec<Instruction>,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Choice { options, .. } => {
                let options: Vec<String> = options
                    .iter()
                    .map(|arm| {
                        let mut out = format!("'{}'->{}", arm.text, arm.target);
                        if let Some(gate) = &arm.gate {
                            out.push_str(&format!(" {}", gate));
                        }
                        if let Some(image) = &arm.image {
                            out.push_str(&format!(" image {}", image));
                        }
                        if let Some(preview) = &arm.preview {
                            out.push_str(&format!(" preview {}", preview));
                        }
                        out
                    })
                    .collect();
                write!(f, "CHOICE [{}]", options.join(", "))
            }
            Instruction::JumpIfFalse {
                condition,
                jump_to_index,
            } => write!(f, "JUMP_IF_FALSE {} (goto {})", condition, jump_to_index),
            Instruction::Goto(index) => write!(f, "GOTO {}", index),
            Instruction::Pause => write!(f, "PAUSE"),
            Instruction::Say {
                char_id: Some(name),
                text,
            } => write!(f, "SAY [{}]: \"{}\"", name, text),
            Instruction::Say {
                char_id: None,
                text,
            } => write!(f, "SAY [NARRATOR]: \"{}\"", text),
            Instruction::Show {
                char_id,
                img_id,
                position: Some(position),
            } => write!(f, "SHOW {} {} AT {}", char_id, img_id, position),
            Instruction::Show {
                char_id, img_id, ..
            } => write!(f, "SHOW {} {}", char_id, img_id),
            Instruction::Background { image } => {
                write!(f, "BACKGROUND {}", image.as_deref().unwrap_or("none"))
            }
            Instruction::Music { track } => {
                write!(f, "MUSIC {}", track.as_deref().unwrap_or("none"))
            }
            Instruction::Sound { id } => write!(f, "SOUND {}", id),
            Instruction::Voice { id } => write!(f, "VOICE {}", id),
            Instruction::Jump { scene_id } => write!(f, "JUMP_SCENE '{}'", scene_id),
            Instruction::End => write!(f, "END"),
            Instruction::Commit => write!(f, "COMMIT"),
            Instruction::With(transition) => write!(f, "WITH {}", transition),
            Instruction::Hide { char_id } => write!(f, "HIDE {}", char_id),
            Instruction::Clear => write!(f, "CLEAR"),
            Instruction::Set { var_id, value } => {
                write!(f, "SET {} = {}", var_id, value.literal())
            }
            Instruction::Add { var_id, amount } => write!(f, "ADD {} {:+}", var_id, amount),
            Instruction::Call { command, args } => {
                write!(f, "CALL {} {}", command, args.join(" "))
            }
        }
    }
}

impl Program {
    pub fn listing(&self) -> String {
        let scene_starts: HashMap<usize, &str> = self
            .scenes
            .iter()
            .map(|(id, &start)| (start, id.as_str()))
            .collect();

        let mut out = String::new();
        for (index, instruction) in self.instructions.iter().enumerate() {
            if let Some(scene) = scene_starts.get(&index) {
                let location = self
                    .scene_locations
                    .get(*scene)
                    .copied()
                    .unwrap_or_default();
                let head = match self.scene_mode(scene) {
                    SceneMode::Adv => scene.to_string(),
                    mode => format!("{} {}", scene, mode),
                };
                match self.file_name(location.file).filter(|f| !f.is_empty()) {
                    Some(file) => out.push_str(&format!(
                        "\nscene {}:  ({}:{})\n",
                        head, file, location.line
                    )),
                    None => out.push_str(&format!("\nscene {}:  (line {})\n", head, location.line)),
                }
            }
            out.push_str(&format!("{:03}: {}\n", index, instruction));
        }
        out
    }
}
