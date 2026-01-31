use std::{collections::HashMap, fs};

use crate::types::{Action, Position, Scene};

pub struct CurrentVisuals<E> {
    pub expression: E,
    pub position: Position,
}

pub struct StoryProvider {
    pub current_scene: Scene,
    pub current_index: usize,
    pub active_characters: HashMap<String, CurrentVisuals<String>>,
}

impl StoryProvider {
    pub fn new(first_scene: &str) -> Self {
        let scene = Self::load(first_scene);

        let mut provider = Self {
            current_scene: scene,
            current_index: 0,
            active_characters: HashMap::new(),
        };

        provider.update_state();
        provider
    }

    fn load(path: &str) -> Scene {
        let bytes = fs::read(path).unwrap_or_else(|e| {
            panic!("❌ Could not read script file at {}: {}", path, e);
        });
        postcard::from_bytes(&bytes)
            .expect("❌ Binary decoding failed. Is the file a valid postcard .bin?")
    }

    pub fn next_line(&mut self) {
        if self.current_index < self.current_scene.lines.len() - 1 {
            self.current_index += 1;
            self.update_state(); // CRITICAL: Update state every line!
        } else if let Some(next) = &self.current_scene.next_scene {
            self.current_scene = Self::load(next);
            self.current_index = 0;
            self.active_characters.clear();
            self.update_state();
        }
    }

    fn update_state(&mut self) {
        let line = &self.current_scene.lines[self.current_index];

        match line.action {
            Action::Enter | Action::Speak => {
                if let Some(char_name) = &line.character {
                    let state =
                        self.active_characters
                            .entry(char_name.clone())
                            .or_insert(CurrentVisuals {
                                expression: line
                                    .expression
                                    .clone()
                                    .unwrap_or_else(|| "default".to_string()),
                                position: Position::Middle,
                            });

                    if let Some(expr) = &line.expression {
                        state.expression = expr.clone();
                    }
                    if let Some(pos) = &line.position {
                        state.position = pos.clone();
                    }
                }
            }
            Action::Leave => {
                if let Some(char_name) = &line.character {
                    self.active_characters.remove(char_name);
                }
            }
            Action::Clear => self.active_characters.clear(),
        }
    }

    pub fn get_render_data(&self) -> Vec<(String, Position)> {
        self.active_characters
            .iter()
            .map(|(name, visuals)| {
                let path =
                    format!("assets/characters/{}/{}.png", name, visuals.expression).to_lowercase();
                (path, visuals.position.clone())
            })
            .collect()
    }
}
