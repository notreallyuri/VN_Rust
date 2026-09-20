use std::collections::HashMap;

use raylib::color::Color;
use vn_script::{CharacterDef, StoryVm, interpolate};

#[derive(Clone, Debug, PartialEq)]
pub struct Character {
    pub name: String,
    pub color: Option<Color>,
    pub images: Vec<String>,
}

impl Character {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: None,
            images: Vec::new(),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn images<I, S>(mut self, images: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.images = images.into_iter().map(Into::into).collect();
        self
    }

    pub fn definition(&self) -> CharacterDef {
        CharacterDef {
            name: self.name.clone(),
            images: self.images.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Characters {
    characters: HashMap<String, Character>,
}

impl Characters {
    pub fn insert(&mut self, id: impl Into<String>, character: Character) {
        self.characters.insert(id.into(), character);
    }

    pub fn get(&self, id: &str) -> Option<&Character> {
        self.characters.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Character)> {
        self.characters.iter().map(|(id, c)| (id.as_str(), c))
    }

    pub fn display_name(&self, speaker: &str, story: &StoryVm) -> String {
        match self.characters.get(speaker) {
            Some(character) => interpolate(&character.name, story.variables()),
            None => speaker.to_string(),
        }
    }

    pub fn color(&self, speaker: &str) -> Option<Color> {
        self.characters.get(speaker).and_then(|c| c.color)
    }
}
