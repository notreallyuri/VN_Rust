use crate::script::types::Node;

use super::types::Instruction;

#[derive(Default)]
pub struct Compiler {
    instructions: Vec<Instruction>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    pub fn compile(mut self, scene_node: Node) -> Vec<Instruction> {
        if let Node::Scene { body, .. } = scene_node {
            for node in body {
                self.compile_node(node);
            }
        }
        self.instructions
    }

    fn compile_node(&mut self, node: Node) {
        match node {
            Node::Show { character, image } => {
                self.instructions.push(Instruction::Show {
                    char_id: character,
                    img_id: image,
                });
            }
            Node::Dialogue { speaker, text } => {
                self.instructions.push(Instruction::Say {
                    char_id: speaker,
                    text,
                });
                self.instructions.push(Instruction::Pause);
            }
            Node::Jump { target } => self
                .instructions
                .push(Instruction::Jump { scene_id: target }),
            Node::ChoiceBlock { options } => {
                let choice_instr_index = self.instructions.len();
                self.instructions.push(Instruction::Pause);

                let mut options_map = Vec::new();
                let mut exit_jumps = Vec::new();

                for opt in options {
                    let start_index = self.instructions.len();
                    options_map.push((opt.text, start_index));

                    for child in opt.body {
                        self.compile_node(child);
                    }

                    exit_jumps.push(self.instructions.len());
                    self.instructions.push(Instruction::Goto(0));
                }

                let end_index = self.instructions.len();

                for idx in exit_jumps {
                    self.instructions[idx] = Instruction::Goto(end_index);
                }

                self.instructions[choice_instr_index] = Instruction::Choice {
                    options: options_map,
                };
            }
            Node::If {
                condition: _,
                then_branch,
                else_branch,
            } => {
                let check_index = self.instructions.len();
                self.instructions.push(Instruction::Pause);

                for child in then_branch {
                    self.compile_node(child);
                }

                let jump_over_else_index = self.instructions.len();
                self.instructions.push(Instruction::Goto(0));

                let else_start_index = self.instructions.len();

                for child in else_branch {
                    self.compile_node(child);
                }

                let end_index = self.instructions.len();

                // TODO: Replace placeholders with real fields from 'condition'
                self.instructions[check_index] = Instruction::JumpIfFalse {
                    var_id: "TODO_VAR".to_string(),
                    op: crate::script::types::Comparison::Equal,
                    value: crate::script::types::Value::Bool(true),
                    jump_to_index: else_start_index,
                };

                self.instructions[jump_over_else_index] = Instruction::Goto(end_index);
            }
            _ => {}
        }
    }
}
