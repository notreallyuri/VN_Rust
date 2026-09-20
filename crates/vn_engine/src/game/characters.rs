use std::collections::HashMap;

use raylib::color::Color;
use vn_script::{CharacterDef, StoryVm, interpolate};

use crate::screens::playing::{BoxOverride, DialogueBoxStyle};

#[derive(Clone, Debug, PartialEq)]
pub struct Character {
    pub name: String,
    pub color: Option<Color>,
    pub images: Vec<String>,
    pub bust: Option<String>,
    pub box_style: Option<BoxOverride>,
}

impl Character {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: None,
            images: Vec::new(),
            bust: None,
            box_style: None,
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

    pub fn bust(mut self, file: impl Into<String>) -> Self {
        self.bust = Some(file.into());
        self
    }

    pub fn box_style(
        mut self,
        style: impl Fn(DialogueBoxStyle) -> DialogueBoxStyle + 'static,
    ) -> Self {
        self.box_style = Some(BoxOverride::new(style));
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

    pub fn bust(&self, speaker: &str) -> Option<&str> {
        self.characters.get(speaker)?.bust.as_deref()
    }

    pub fn dialogue_box(&self, speaker: Option<&str>, base: &DialogueBoxStyle) -> DialogueBoxStyle {
        match speaker.and_then(|id| self.characters.get(id)) {
            Some(character) => match &character.box_style {
                Some(style) => style.apply(base),
                None => base.clone(),
            },
            None => base.clone(),
        }
    }
}
