use std::collections::HashMap;

use crate::diagnostics::Diagnostic;
use crate::types::{Instruction, Location, Node, Program, Stmt};
use crate::{parse_program, tokenize};

pub fn compile_source(source: &str) -> Program {
    let mut compiler = Compiler::new();
    let diagnostics = compiler.parse_and_compile(source);
    compiler.finish_with(diagnostics)
}

pub fn compile_sources<N, S>(sources: impl IntoIterator<Item = (N, S)>) -> Program
where
    N: Into<String>,
    S: AsRef<str>,
{
    let mut compiler = Compiler::new();
    let mut diagnostics = Vec::new();

    for (name, source) in sources {
        let name = name.into();
        compiler.start_file(name.clone());
        diagnostics.extend(
            compiler
                .parse_and_compile(source.as_ref())
                .into_iter()
                .map(|d| d.with_file(Some(&name))),
        );
    }

    compiler.finish_with(diagnostics)
}

#[derive(Default)]
pub struct Compiler {
    instructions: Vec<Instruction>,
    locations: Vec<Location>,
    files: Vec<String>,
    scenes: HashMap<String, usize>,
    scene_order: Vec<String>,
    scene_locations: HashMap<String, Location>,
    diagnostics: Vec<Diagnostic>,
    file: usize,
    line: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compile(mut self, scenes: Vec<Stmt>) -> Program {
        self.compile_scenes(scenes);
        self.finish()
    }

    pub fn start_file(&mut self, name: impl Into<String>) {
        self.file = self.files.len();
        self.files.push(name.into());
    }

    pub fn compile_scenes(&mut self, scenes: Vec<Stmt>) {
        for scene in scenes {
            if let Node::Scene { id, body } = scene.node {
                self.line = scene.line;
                self.compile_scene(id, body);
            }
        }
    }

    pub fn finish(self) -> Program {
        let mut program = Program {
            instructions: self.instructions,
            locations: self.locations,
            files: self.files,
            scenes: self.scenes,
            scene_order: self.scene_order,
            scene_locations: self.scene_locations,
            diagnostics: Vec::new(),
        };

        let mut diagnostics = self.diagnostics;
        program.sort_diagnostics(&mut diagnostics);
        program.diagnostics = diagnostics;
        program
    }

    fn parse_and_compile(&mut self, source: &str) -> Vec<Diagnostic> {
        let (tokens, mut diagnostics) = tokenize(source);
        let (scenes, parse_diagnostics) = parse_program(&tokens);
        diagnostics.extend(parse_diagnostics);
        self.compile_scenes(scenes);
        diagnostics
    }

    fn finish_with(mut self, diagnostics: Vec<Diagnostic>) -> Program {
        self.diagnostics.extend(diagnostics);
        self.finish()
    }

    fn here_location(&self) -> Location {
        Location {
            file: self.file,
            line: self.line,
        }
    }

    fn compile_scene(&mut self, id: String, body: Vec<Stmt>) {
        let location = self.here_location();

        if let Some(first) = self.scene_locations.get(&id) {
            let place = match self.files.get(first.file) {
                Some(file) if first.file != self.file => format!("{}:{}", file, first.line),
                _ => format!("line {}", first.line),
            };
            let diagnostic = Diagnostic::error(
                self.line,
                format!(
                    "scene '{}' is already defined at {}; this one is ignored",
                    id, place
                ),
            )
            .with_file(self.files.get(self.file).map(String::as_str));
            self.diagnostics.push(diagnostic);
            return;
        }

        self.scenes.insert(id.clone(), self.instructions.len());
        self.scene_locations.insert(id.clone(), location);
        self.scene_order.push(id);

        self.compile_body(body);

        self.line = location.line;
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
        self.locations.push(self.here_location());
        self.instructions.len() - 1
    }

    fn emit_transition(&mut self, transition: Option<crate::Transition>) {
        if let Some(transition) = transition {
            self.emit(Instruction::With(transition));
        }
    }

    fn compile_node(&mut self, node: Node) {
        match node {
            Node::Show {
                character,
                image,
                position,
                transition,
            } => {
                self.emit_transition(transition);
                self.emit(Instruction::Show {
                    char_id: character,
                    img_id: image,
                    position,
                });
            }
            Node::Background { image, transition } => {
                self.emit_transition(transition);
                self.emit(Instruction::Background { image });
            }
            Node::Music { track } => {
                self.emit(Instruction::Music { track });
            }
            Node::Sound { id } => {
                self.emit(Instruction::Sound { id });
            }
            Node::Voice { id } => {
                self.emit(Instruction::Voice { id });
            }
            Node::Remove {
                character,
                transition,
            } => {
                self.emit_transition(transition);
                self.emit(Instruction::Hide { char_id: character });
            }
            Node::Clear { transition } => {
                self.emit_transition(transition);
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
