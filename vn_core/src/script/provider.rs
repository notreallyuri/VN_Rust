use std::collections::HashMap;

use crate::script::{Comparison, Compiler, Instruction, TokenKind, Value, parse_scene, tokenize};

pub struct StoryProvider {
    pub instructions: Vec<Instruction>,
    pub ip: usize,
    pub variables: HashMap<String, Value>,
    pub active_characters: HashMap<String, String>,
}

impl StoryProvider {
    pub fn from_source(source: &str) -> Self {
        let tokens = tokenize(source);
        let mut all_instructions = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            if tokens[i].kind == TokenKind::Scene {
                let (scene_node, next_i) = parse_scene(&tokens, i);
                let compiler = Compiler::new();
                all_instructions.extend(compiler.compile(scene_node));
                i = next_i;
            } else {
                i += 1;
            }
        }

        Self {
            instructions: all_instructions,
            ip: 0,
            variables: HashMap::new(),
            active_characters: HashMap::new(),
        }
    }

    pub fn step(&mut self) {
        if self.ip >= self.instructions.len() {
            println!("📝 End of script reached at IP: {}", self.ip);
            return;
        }

        let instr = &self.instructions[self.ip];

        match instr {
            Instruction::Show { char_id, img_id } => {
                self.active_characters
                    .insert(char_id.clone(), img_id.clone());
                self.ip += 1;
                self.step();
            }
            Instruction::Hide { char_id } => {
                self.active_characters.remove(char_id);
                self.ip += 1;
                self.step();
            }
            Instruction::Clear => {
                self.active_characters.clear();
                self.ip += 1;
                self.step();
            }
            Instruction::Set { var_id, value } => {
                self.variables.insert(var_id.clone(), value.clone());
                self.ip += 1;
                self.step();
            }
            Instruction::Add { var_id, amount } => {
                let entry = self
                    .variables
                    .entry(var_id.clone())
                    .or_insert(Value::Int(0));

                if let Value::Int(val) = entry {
                    *val += *amount;
                }
                self.ip += 1;
                self.step();
            }
            Instruction::Goto(target) => {
                self.ip = *target;
                self.step(); // Immediate jump
            }
            Instruction::JumpIfFalse {
                var_id,
                op,
                value,
                jump_to_index,
            } => {
                let var_val = self.variables.get(var_id);

                let condition = match (var_val, value) {
                    (Some(Value::Int(a)), Value::Int(b)) => match op {
                        Comparison::Gt => a > b,
                        Comparison::Lt => a < b,
                        Comparison::Equal => a == b,
                        Comparison::NotEqual => a != b,
                        Comparison::Gte => a >= b,
                        Comparison::Lte => a <= b,
                    },
                    (Some(Value::Bool(a)), Value::Bool(b)) => a == b,
                    _ => false,
                };

                if !condition {
                    self.ip = *jump_to_index;
                } else {
                    self.ip += 1;
                }
                self.step();
            }
            Instruction::Say { .. } | Instruction::Pause | Instruction::Choice { .. } => {
                // TODO: Finish this
            }
            Instruction::Jump { scene_id: _ } => {
                // TODO: Also finish this
            }
            _ => {
                self.ip += 1;
            }
        }
    }

    pub fn next(&mut self) {
        self.ip += 1;
        self.step();
    }
}
