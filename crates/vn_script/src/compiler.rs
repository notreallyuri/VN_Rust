use std::collections::HashMap;

use crate::diagnostics::Diagnostic;
use crate::types::{Instruction, Node, Program, Stmt};
use crate::{parse_program, tokenize};

pub fn compile_source(source: &str) -> Program {
    let (tokens, mut diagnostics) = tokenize(source);
    let (scenes, parse_diagnostics) = parse_program(&tokens);
    diagnostics.extend(parse_diagnostics);

    let mut program = Compiler::new().compile(scenes);
    diagnostics.append(&mut program.diagnostics);
    diagnostics.sort_by_key(|d| d.line);
    program.diagnostics = diagnostics;
    program
}

#[derive(Default)]
pub struct Compiler {
    instructions: Vec<Instruction>,
    lines: Vec<usize>,
    scenes: HashMap<String, usize>,
    scene_order: Vec<String>,
    scene_lines: HashMap<String, usize>,
    diagnostics: Vec<Diagnostic>,
    line: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile(mut self, scenes: Vec<Stmt>) -> Program {
        for scene in scenes {
            if let Node::Scene { id, body } = scene.node {
                self.line = scene.line;
                self.compile_scene(id, body);
            }
        }

        Program {
            instructions: self.instructions,
            lines: self.lines,
            scenes: self.scenes,
            scene_order: self.scene_order,
            scene_lines: self.scene_lines,
            diagnostics: self.diagnostics,
        }
    }

    fn compile_scene(&mut self, id: String, body: Vec<Stmt>) {
        if let Some(first) = self.scene_lines.get(&id) {
            self.diagnostics.push(Diagnostic::error(
                self.line,
                format!(
                    "scene '{}' is already defined at line {}; this one is ignored",
                    id, first
                ),
            ));
            return;
        }

        let scene_line = self.line;
        self.scenes.insert(id.clone(), self.instructions.len());
        self.scene_lines.insert(id.clone(), scene_line);
        self.scene_order.push(id);

        self.compile_body(body);

        self.line = scene_line;
        self.emit(Instruction::End);
    }

    fn compile_body(&mut self, body: Vec<Stmt>) {
        for stmt in body {
            self.line = stmt.line;
            self.compile_node(stmt.node);
        }
    }

    fn here(&self) -> usize {
        self.instructions.len()
    }

    fn emit(&mut self, instruction: Instruction) -> usize {
        self.instructions.push(instruction);
        self.lines.push(self.line);
        self.instructions.len() - 1
    }

    fn compile_node(&mut self, node: Node) {
        match node {
            Node::Show { character, image } => {
                self.emit(Instruction::Show {
                    char_id: character,
                    img_id: image,
                });
            }
            Node::Remove { character } => {
                self.emit(Instruction::Hide { char_id: character });
            }
            Node::Clear => {
                self.emit(Instruction::Clear);
            }
            Node::Commit => {
                self.emit(Instruction::Commit);
            }
            Node::Dialogue { speaker, text } => {
                self.emit(Instruction::Say {
                    char_id: speaker,
                    text,
                });
            }
            Node::Jump { target } => {
                self.emit(Instruction::Jump { scene_id: target });
            }
            Node::Call { command, args } => {
                self.emit(Instruction::Call { command, args });
            }
            Node::Set { var_id, value } => {
                self.emit(Instruction::Set { var_id, value });
            }
            Node::Add { var_id, amount } => {
                self.emit(Instruction::Add { var_id, amount });
            }
            Node::ChoiceBlock {
                options,
                final_choice,
            } => {
                let choice_line = self.line;
                let choice_index = self.emit(Instruction::Pause);

                let mut options_map = Vec::new();
                let mut exit_jumps = Vec::new();

                for option in options {
                    options_map.push((option.text, self.here()));

                    if final_choice {
                        self.line = option.line;
                        self.emit(Instruction::Commit);
                    }

                    self.compile_body(option.body);

                    self.line = option.line;
                    exit_jumps.push(self.emit(Instruction::Goto(0)));
                }

                let end_index = self.here();
                for index in exit_jumps {
                    self.instructions[index] = Instruction::Goto(end_index);
                }

                self.instructions[choice_index] = Instruction::Choice {
                    options: options_map,
                };
                self.line = choice_line;
            }
            Node::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let if_line = self.line;
                let check_index = self.emit(Instruction::Pause);

                self.compile_body(then_branch);

                self.line = if_line;
                let jump_over_else = self.emit(Instruction::Goto(0));
                let else_start = self.here();

                self.compile_body(else_branch);

                self.instructions[check_index] = Instruction::JumpIfFalse {
                    condition,
                    jump_to_index: else_start,
                };
                self.instructions[jump_over_else] = Instruction::Goto(self.here());
                self.line = if_line;
            }
            Node::Scene { .. } => {}
        }
    }
}
