use std::collections::HashMap;

use super::{Event, StoryVm, VmError};
use crate::{Comparison, Condition, Instruction, Value, interpolate};

const MAX_SILENT_STEPS: usize = 100_000;

impl StoryVm {
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

    pub(super) fn next_event(&mut self) -> Event {
        if let Some(choice_ip) = self.pending_choice {
            return self.choice_event(choice_ip);
        }

        for _ in 0..MAX_SILENT_STEPS {
            if std::mem::take(&mut self.entered)
                && self.scene_events
                && let Some(scene) = &self.current_scene
            {
                return Event::SceneEnter {
                    scene: scene.clone(),
                };
            }

            let Some(instr) = self.program.instructions.get(self.ip) else {
                return Event::End;
            };

            match instr {
                Instruction::Say { char_id, text } => {
                    let event = Event::Say {
                        speaker: char_id
                            .as_deref()
                            .map(|speaker| interpolate(speaker, &self.variables)),
                        text: interpolate(self.localized(self.ip, text), &self.variables),
                    };
                    self.ip += 1;
                    return event;
                }
                Instruction::Choice { options } if options.is_empty() => self.ip += 1,
                Instruction::Choice { .. } => {
                    self.pending_choice = Some(self.ip);
                    return self.choice_event(self.ip);
                }
                Instruction::Show {
                    char_id,
                    img_id,
                    position,
                } => {
                    let event = Event::Show {
                        character: char_id.clone(),
                        image: img_id.clone(),
                        position: *position,
                        transition: self.pending_transition.take(),
                    };
                    self.active_characters
                        .insert(char_id.clone(), img_id.clone());
                    if let Some(position) = position {
                        self.positions.insert(char_id.clone(), *position);
                    }
                    self.ip += 1;
                    return event;
                }
                Instruction::Background { image } => {
                    let event = Event::Background {
                        image: image.clone(),
                        transition: self.pending_transition.take(),
                    };
                    self.background = image.clone();
                    self.ip += 1;
                    return event;
                }
                Instruction::Music { track } => {
                    let event = Event::Music {
                        track: track.clone(),
                    };
                    self.music = track.clone();
                    self.ip += 1;
                    return event;
                }
                Instruction::Sound { id } => {
                    let event = Event::Sound { id: id.clone() };
                    self.ip += 1;
                    return event;
                }
                Instruction::Voice { id } => {
                    let event = Event::Voice { id: id.clone() };
                    self.ip += 1;
                    return event;
                }
                Instruction::Hide { char_id } => {
                    let event = Event::Hide {
                        character: char_id.clone(),
                        transition: self.pending_transition.take(),
                    };
                    self.active_characters.remove(char_id);
                    self.positions.remove(char_id);
                    self.ip += 1;
                    return event;
                }
                Instruction::Clear => {
                    self.active_characters.clear();
                    self.positions.clear();
                    self.ip += 1;
                    return Event::Clear {
                        transition: self.pending_transition.take(),
                    };
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
                Instruction::With(transition) => {
                    self.pending_transition = Some(*transition);
                    self.ip += 1;
                }
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

    pub fn set_scene_events(&mut self, enabled: bool) {
        self.scene_events = enabled;
    }

    pub(super) fn reset_position(&mut self) {
        match self.entry_scene().map(str::to_string) {
            Some(entry) => self.enter_scene(&entry).expect("entry scene exists"),
            None => {
                self.current_scene = None;
                self.ip = self.program.instructions.len();
            }
        }
    }

    pub(super) fn enter_scene(&mut self, scene_id: &str) -> Result<(), VmError> {
        let start = *self
            .program
            .scenes
            .get(scene_id)
            .ok_or_else(|| VmError::UnknownScene(scene_id.to_string()))?;

        self.ip = start;
        self.current_scene = Some(scene_id.to_string());
        self.entered = true;
        Ok(())
    }

    pub(super) fn choice_event(&self, choice_ip: usize) -> Event {
        let Some(Instruction::Choice { options }) = self.program.instructions.get(choice_ip) else {
            unreachable!("pending_choice always points at a Choice");
        };

        Event::Choice {
            options: options
                .iter()
                .map(|(text, _)| interpolate(self.localized(choice_ip, text), &self.variables))
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
