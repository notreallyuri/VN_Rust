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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    Show {
        char_id: String,
        img_id: String,
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
        options: Vec<(String, usize)>,
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
            Instruction::Choice { options } => Instruction::Choice {
                options: options
                    .iter()
                    .map(|(text, target)| (text.clone(), shift(*target)))
                    .collect(),
            },
            other => other.clone(),
        }
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, &byte| {
        (hash ^ byte as u64).wrapping_mul(0x100000001b3)
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneInstruction {
    pub id: String,
    pub instructions: Vec<Instruction>,
}
