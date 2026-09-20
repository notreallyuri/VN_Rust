use std::collections::BTreeMap;

use vn_engine::data::assets::Assets;
use vn_engine::raylib::prelude::Vector2;

use crate::{Error, ModelAssets};

#[derive(Clone, Debug, Default)]
pub struct Appearance {
    pub(crate) expression: Option<String>,
    pub(crate) motion: Option<(String, usize, bool)>,
    pub(crate) parameters: BTreeMap<String, f32>,
}

impl Appearance {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expression(mut self, name: impl Into<String>) -> Self {
        self.expression = Some(name.into());
        self
    }

    pub fn motion(mut self, group: impl Into<String>, index: usize, looping: bool) -> Self {
        self.motion = Some((group.into(), index, looping));
        self
    }

    pub fn parameter(mut self, id: impl Into<String>, value: f32) -> Self {
        self.parameters.insert(id.into(), value);
        self
    }
}

#[derive(Clone, Debug)]
pub struct Live2dCharacter {
    pub(crate) model: String,
    pub(crate) appearances: BTreeMap<String, Appearance>,
    pub(crate) size: Option<Vector2>,
}

impl Live2dCharacter {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            appearances: BTreeMap::new(),
            size: None,
        }
    }

    pub fn appearance(mut self, name: impl Into<String>, appearance: Appearance) -> Self {
        self.appearances.insert(name.into(), appearance);
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Some(Vector2::new(width, height));
        self
    }

    pub fn check(&self, assets: &Assets) -> Result<(), Error> {
        self.bundle(assets).map(|_| ())
    }

    pub(crate) fn bundle(&self, assets: &Assets) -> Result<ModelAssets, Error> {
        if self.appearances.is_empty() {
            return Err(Error(
                "Live2D character needs at least one named appearance".into(),
            ));
        }
        if self.size.is_some_and(|size| {
            !size.x.is_finite() || !size.y.is_finite() || size.x <= 0.0 || size.y <= 0.0
        }) {
            return Err(Error(
                "Live2D character size must be finite and positive".into(),
            ));
        }
        let bundle = ModelAssets::load(assets, &self.model)?;
        let refs = &bundle.settings().file_references;
        for (name, appearance) in &self.appearances {
            if !vn_engine::script::is_identifier(name) {
                return Err(Error(format!("invalid appearance name '{name}'")));
            }
            if let Some(expression) = &appearance.expression
                && !refs
                    .expressions
                    .iter()
                    .any(|entry| &entry.name == expression)
            {
                return Err(Error(format!(
                    "appearance '{name}': unknown expression '{expression}'"
                )));
            }
            if let Some((group, index, _)) = &appearance.motion
                && refs
                    .motions
                    .get(group)
                    .and_then(|motions| motions.get(*index))
                    .is_none()
            {
                return Err(Error(format!(
                    "appearance '{name}': unknown motion '{group}'[{index}]"
                )));
            }
            for (id, value) in &appearance.parameters {
                if id.is_empty() || id.contains('\0') || !value.is_finite() {
                    return Err(Error(format!(
                        "appearance '{name}': invalid parameter '{id}'"
                    )));
                }
            }
        }
        Ok(bundle)
    }
}
