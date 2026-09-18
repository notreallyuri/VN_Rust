use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    Comparison, Compiler, Condition, Diagnostic, Instruction, Program, Schema, Value, VarType,
    interpolate, parse_program, tokenize,
};

const MAX_SILENT_STEPS: usize = 100_000;

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
    },
    Hide {
        character: String,
    },
    Clear,
    Call {
        command: String,
        args: Vec<String>,
    },
    Commit,
    End,
}

impl Event {
    pub fn is_blocking(&self) -> bool {
        matches!(self, Event::Say { .. } | Event::Choice { .. } | Event::End)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    UnknownScene(String),
    NoChoicePending,
    ChoiceOutOfRange { index: usize, options: usize },
    UnknownVariable(String),
    TypeMismatch { variable: String, message: String },
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::UnknownScene(id) => write!(f, "no scene named '{}'", id),
            VmError::NoChoicePending => write!(f, "no choice is waiting for an answer"),
            VmError::ChoiceOutOfRange { index, options } => {
                write!(f, "choice {} is out of range ({} options)", index, options)
            }
            VmError::UnknownVariable(id) => write!(f, "unknown variable '{}'", id),
            VmError::TypeMismatch { variable, message } => {
                write!(f, "variable '{}': {}", variable, message)
            }
        }
    }
}

impl std::error::Error for VmError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorySnapshot {
    pub scene: Option<String>,
    pub offset: usize,
    pub scene_fingerprint: u64,
    pub pending_choice: bool,
    pub current: Option<Event>,
    pub variables: BTreeMap<String, Value>,
    pub active_characters: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreOutcome {
    Exact,
    SceneRestarted { scene: String },
}

#[derive(Debug)]
pub struct StoryVm {
    program: Program,
    ip: usize,
    current_scene: Option<String>,
    variables: HashMap<String, Value>,
    active_characters: HashMap<String, String>,
    pending_choice: Option<usize>,
    current: Option<Event>,
    schema: Schema,
    entry: Option<String>,
}

impl StoryVm {
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let source = std::fs::read_to_string(path)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", path.display(), e)))?;
        Ok(Self::from_source(&source))
    }

    pub fn from_source(source: &str) -> Self {
        let scenes = parse_program(&tokenize(source));
        Self::from_program(Compiler::new().compile(scenes))
    }

    pub fn from_program(program: Program) -> Self {
        let mut vm = Self {
            program,
            ip: 0,
            current_scene: None,
            variables: HashMap::new(),
            active_characters: HashMap::new(),
            pending_choice: None,
            current: None,
            schema: Schema::default(),
            entry: None,
        };
        vm.reset();
        vm
    }

    pub fn set_schema(&mut self, schema: Schema) {
        self.schema = schema;
        self.reset();
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn validate(&self) -> Vec<Diagnostic> {
        let mut diagnostics = self.schema.validate(&self.program);

        if let Some(entry) = &self.entry
            && !self.program.scenes.contains_key(entry)
        {
            diagnostics.insert(
                0,
                Diagnostic::error(0, format!("entry scene '{}' does not exist", entry)),
            );
        }

        diagnostics
    }

    pub fn set_entry_scene(&mut self, scene_id: impl Into<String>) -> Result<(), VmError> {
        let scene_id = scene_id.into();
        if !self.program.scenes.contains_key(&scene_id) {
            return Err(VmError::UnknownScene(scene_id));
        }

        self.entry = Some(scene_id);
        self.reset();
        Ok(())
    }

    pub fn entry_scene(&self) -> Option<&str> {
        self.entry.as_deref().or_else(|| self.program.entry_scene())
    }

    pub fn reset(&mut self) {
        self.variables = self.schema.defaults().into_iter().collect();
        self.active_characters.clear();
        self.pending_choice = None;
        self.current = None;
        self.reset_position();
    }

    pub fn start_at(&mut self, scene_id: &str) -> Result<(), VmError> {
        if !self.program.scenes.contains_key(scene_id) {
            return Err(VmError::UnknownScene(scene_id.to_string()));
        }

        self.reset();
        self.enter_scene(scene_id)
    }

    pub fn advance(&mut self) -> Event {
        let event = self.next_event();
        self.current = event.is_blocking().then(|| event.clone());
        event
    }

    pub fn current(&self) -> Option<&Event> {
        self.current.as_ref()
    }

    fn next_event(&mut self) -> Event {
        if let Some(choice_ip) = self.pending_choice {
            return self.choice_event(choice_ip);
        }

        for _ in 0..MAX_SILENT_STEPS {
            let Some(instr) = self.program.instructions.get(self.ip) else {
                return Event::End;
            };

            match instr {
                Instruction::Say { char_id, text } => {
                    let event = Event::Say {
                        speaker: char_id
                            .as_deref()
                            .map(|speaker| interpolate(speaker, &self.variables)),
                        text: interpolate(text, &self.variables),
                    };
                    self.ip += 1;
                    return event;
                }
                Instruction::Choice { options } if options.is_empty() => self.ip += 1,
                Instruction::Choice { .. } => {
                    self.pending_choice = Some(self.ip);
                    return self.choice_event(self.ip);
                }
                Instruction::Show { char_id, img_id } => {
                    let event = Event::Show {
                        character: char_id.clone(),
                        image: img_id.clone(),
                    };
                    self.active_characters
                        .insert(char_id.clone(), img_id.clone());
                    self.ip += 1;
                    return event;
                }
                Instruction::Hide { char_id } => {
                    let event = Event::Hide {
                        character: char_id.clone(),
                    };
                    self.active_characters.remove(char_id);
                    self.ip += 1;
                    return event;
                }
                Instruction::Clear => {
                    self.active_characters.clear();
                    self.ip += 1;
                    return Event::Clear;
                }
                Instruction::Call { command, args } => {
                    let event = Event::Call {
                        command: command.clone(),
                        args: args.clone(),
                    };
                    self.ip += 1;
                    return event;
                }
                Instruction::End => return Event::End,
                Instruction::Jump { scene_id } => {
                    let scene_id = scene_id.clone();
                    if let Err(e) = self.enter_scene(&scene_id) {
                        eprintln!("⚠️ Ending the story: {}", e);
                        self.ip = self.program.instructions.len();
                        return Event::End;
                    }
                }
                Instruction::Goto(target) => self.ip = *target,
                Instruction::JumpIfFalse {
                    condition,
                    jump_to_index,
                } => {
                    self.ip = if evaluate(condition, &self.variables) {
                        self.ip + 1
                    } else {
                        *jump_to_index
                    };
                }
                Instruction::Set { var_id, value } => {
                    self.variables.insert(var_id.clone(), value.clone());
                    self.ip += 1;
                }
                Instruction::Add { var_id, amount } => {
                    match self
                        .variables
                        .entry(var_id.clone())
                        .or_insert(Value::Int(0))
                    {
                        Value::Int(val) => *val = val.saturating_add(*amount),
                        other => eprintln!(
                            "⚠️ `add {}`: variable holds {}, not an integer",
                            var_id,
                            other.literal()
                        ),
                    }
                    self.ip += 1;
                }
                Instruction::Pause => self.ip += 1,
                Instruction::Commit => {
                    self.ip += 1;
                    return Event::Commit;
                }
            }
        }

        eprintln!(
            "⚠️ Ending the story: {} steps without any dialogue (a jump loop?)",
            MAX_SILENT_STEPS
        );
        self.ip = self.program.instructions.len();
        Event::End
    }

    pub fn advance_until_blocking(&mut self) -> Event {
        loop {
            let event = self.advance();
            if event.is_blocking() {
                return event;
            }
        }
    }

    pub fn choose(&mut self, index: usize) -> Result<(), VmError> {
        let choice_ip = self.pending_choice.ok_or(VmError::NoChoicePending)?;

        let Some(Instruction::Choice { options }) = self.program.instructions.get(choice_ip) else {
            unreachable!("pending_choice always points at a Choice");
        };

        let (_, target) = options.get(index).ok_or(VmError::ChoiceOutOfRange {
            index,
            options: options.len(),
        })?;

        self.ip = *target;
        self.pending_choice = None;
        self.current = None;
        Ok(())
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn current_scene(&self) -> Option<&str> {
        self.current_scene.as_deref()
    }

    pub fn active_characters(&self) -> &HashMap<String, String> {
        &self.active_characters
    }

    pub fn variables(&self) -> &HashMap<String, Value> {
        &self.variables
    }

    pub fn variable(&self, id: &str) -> Option<&Value> {
        self.variables.get(id)
    }

    pub fn set_variable(&mut self, id: impl Into<String>, value: Value) -> Result<(), VmError> {
        let id = id.into();

        if !self.schema.variables.is_empty() {
            let def = self
                .schema
                .variables
                .get(&id)
                .ok_or_else(|| VmError::UnknownVariable(id.clone()))?;

            def.ty
                .check(&value)
                .map_err(|message| VmError::TypeMismatch {
                    variable: id.clone(),
                    message,
                })?;
        }

        self.variables.insert(id, value);
        Ok(())
    }

    pub fn variable_type(&self, id: &str) -> Option<&VarType> {
        self.schema.variables.get(id).map(|def| &def.ty)
    }

    pub fn snapshot(&self) -> StorySnapshot {
        let scene_start = self
            .current_scene
            .as_deref()
            .and_then(|scene| self.program.scenes.get(scene).copied())
            .unwrap_or(0);

        StorySnapshot {
            scene: self.current_scene.clone(),
            offset: self.ip.saturating_sub(scene_start),
            scene_fingerprint: self
                .current_scene
                .as_deref()
                .and_then(|scene| self.program.scene_fingerprint(scene))
                .unwrap_or(0),
            pending_choice: self.pending_choice.is_some(),
            current: self.current.clone(),
            variables: self
                .variables
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            active_characters: self
                .active_characters
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

    pub fn check_restore(&self, snapshot: &StorySnapshot) -> Result<RestoreOutcome, VmError> {
        let Some(scene) = &snapshot.scene else {
            return Ok(RestoreOutcome::Exact);
        };

        let range = self
            .program
            .scene_range(scene)
            .ok_or_else(|| VmError::UnknownScene(scene.clone()))?;
        let ip = range.start + snapshot.offset;

        let unchanged = self.program.scene_fingerprint(scene) == Some(snapshot.scene_fingerprint)
            && ip < range.end
            && (!snapshot.pending_choice
                || matches!(self.program.instructions[ip], Instruction::Choice { .. }));

        Ok(if unchanged {
            RestoreOutcome::Exact
        } else {
            RestoreOutcome::SceneRestarted {
                scene: scene.clone(),
            }
        })
    }

    pub fn restore(&mut self, snapshot: &StorySnapshot) -> Result<RestoreOutcome, VmError> {
        let outcome = self.check_restore(snapshot)?;

        self.variables = self.schema.defaults().into_iter().collect();
        self.variables.extend(
            snapshot
                .variables
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );
        self.active_characters = snapshot.active_characters.clone().into_iter().collect();
        self.pending_choice = None;
        self.current = None;

        match (&snapshot.scene, &outcome) {
            (None, _) => self.reset_position(),
            (Some(scene), RestoreOutcome::SceneRestarted { .. }) => self.enter_scene(scene)?,
            (Some(scene), RestoreOutcome::Exact) => {
                self.enter_scene(scene)?;
                self.ip += snapshot.offset;
                self.pending_choice = snapshot.pending_choice.then_some(self.ip);
                self.current = snapshot.current.clone();
            }
        }

        Ok(outcome)
    }

    fn reset_position(&mut self) {
        match self.entry_scene().map(str::to_string) {
            Some(entry) => self.enter_scene(&entry).expect("entry scene exists"),
            None => {
                self.current_scene = None;
                self.ip = self.program.instructions.len();
            }
        }
    }

    fn enter_scene(&mut self, scene_id: &str) -> Result<(), VmError> {
        let start = *self
            .program
            .scenes
            .get(scene_id)
            .ok_or_else(|| VmError::UnknownScene(scene_id.to_string()))?;

        self.ip = start;
        self.current_scene = Some(scene_id.to_string());
        Ok(())
    }

    fn choice_event(&self, choice_ip: usize) -> Event {
        let Some(Instruction::Choice { options }) = self.program.instructions.get(choice_ip) else {
            unreachable!("pending_choice always points at a Choice");
        };

        Event::Choice {
            options: options
                .iter()
                .map(|(text, _)| interpolate(text, &self.variables))
                .collect(),
        }
    }
}

fn evaluate(condition: &Condition, variables: &HashMap<String, Value>) -> bool {
    match condition {
        Condition::Test { var_id, op, value } => compare(variables.get(var_id), *op, value),
        Condition::All(parts) => parts.iter().all(|part| evaluate(part, variables)),
        Condition::Any(parts) => parts.iter().any(|part| evaluate(part, variables)),
    }
}

fn compare(actual: Option<&Value>, op: Comparison, expected: &Value) -> bool {
    let actual = match (actual, expected) {
        (Some(actual), _) => actual,
        (None, Value::Bool(_)) => &Value::Bool(false),
        (None, Value::Int(_)) => &Value::Int(0),
        (None, Value::String(_)) => &Value::String(String::new()),
        (None, Value::Enum(_)) => return op == Comparison::NotEqual,
    };

    match (actual, expected) {
        (Value::Int(a), Value::Int(b)) => match op {
            Comparison::Equal => a == b,
            Comparison::NotEqual => a != b,
            Comparison::Gte => a >= b,
            Comparison::Lte => a <= b,
            Comparison::Gt => a > b,
            Comparison::Lt => a < b,
        },
        (Value::Bool(_), Value::Bool(_))
        | (Value::Enum(_), Value::Enum(_))
        | (Value::String(_), Value::String(_)) => match op {
            Comparison::Equal => actual == expected,
            Comparison::NotEqual => actual != expected,
            _ => false,
        },
        _ => false,
    }
}
