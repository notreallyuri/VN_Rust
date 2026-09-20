use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use vn_engine::data::assets::Assets;

use crate::Error;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ModelSettings {
    pub version: u32,
    pub file_references: FileReferences,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileReferences {
    pub moc: String,
    pub textures: Vec<String>,
    pub physics: Option<String>,
    pub pose: Option<String>,
    #[serde(default)]
    pub expressions: Vec<Expression>,
    #[serde(default)]
    pub motions: BTreeMap<String, Vec<Motion>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Expression {
    pub name: String,
    pub file: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Motion {
    pub file: String,
    pub fade_in_time: Option<f32>,
    pub fade_out_time: Option<f32>,
}

/// An owned, preflighted bundle. Every file is read through the engine's `Assets`,
/// so the same loader supports folders and executable-embedded model files.
pub struct ModelAssets {
    pub(crate) settings: ModelSettings,
    pub(crate) manifest: Vec<u8>,
    pub(crate) files: BTreeMap<String, Vec<u8>>,
}

impl ModelAssets {
    pub fn load(assets: &Assets, manifest_path: &str) -> Result<Self, Error> {
        let manifest_path = resolve("", manifest_path)?;
        let manifest = read(assets, &manifest_path)?;
        let settings: ModelSettings = serde_json::from_slice(&manifest)
            .map_err(|e| Error(format!("{manifest_path}: {e}")))?;
        if settings.version != 3 {
            return Err(Error(format!(
                "{manifest_path}: expected model3.json Version 3"
            )));
        }
        let refs = &settings.file_references;
        if refs.textures.is_empty() {
            return Err(Error(format!("{manifest_path}: model has no textures")));
        }
        let mut names = BTreeSet::new();
        for expression in &refs.expressions {
            if expression.name.is_empty()
                || expression.name.contains('\0')
                || !names.insert(&expression.name)
            {
                return Err(Error(format!(
                    "{manifest_path}: invalid or duplicate expression name"
                )));
            }
        }
        for (group, motions) in &refs.motions {
            if group.is_empty() || group.contains('\0') {
                return Err(Error(format!("{manifest_path}: invalid motion group")));
            }
            for motion in motions {
                for fade in [motion.fade_in_time, motion.fade_out_time]
                    .into_iter()
                    .flatten()
                {
                    // Cubism uses -1 for an unspecified fade.
                    if !fade.is_finite() || (fade < 0.0 && fade != -1.0) {
                        return Err(Error(format!("{}: invalid motion fade time", motion.file)));
                    }
                }
            }
        }
        let base = manifest_path.rsplit_once('/').map_or("", |(base, _)| base);
        let mut files = BTreeMap::new();
        let paths = std::iter::once(&refs.moc)
            .chain(&refs.textures)
            .chain(refs.physics.iter())
            .chain(refs.pose.iter())
            .chain(refs.expressions.iter().map(|e| &e.file))
            .chain(refs.motions.values().flatten().map(|m| &m.file));
        for path in paths {
            let resolved = resolve(base, path)?;
            let bytes = read(assets, &resolved)?;
            if path.ends_with(".json") {
                let value: serde_json::Value = serde_json::from_slice(&bytes)
                    .map_err(|e| Error(format!("{resolved}: {e}")))?;
                if !value.is_object() {
                    return Err(Error(format!("{resolved}: expected a JSON object")));
                }
            }
            files.insert(path.clone(), bytes);
        }
        Ok(Self {
            settings,
            manifest,
            files,
        })
    }

    pub fn settings(&self) -> &ModelSettings {
        &self.settings
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn manifest_bytes(&self) -> &[u8] {
        &self.manifest
    }
}

fn read(assets: &Assets, path: &str) -> Result<Vec<u8>, Error> {
    let bytes = assets
        .read(path)
        .map_err(|e| Error(format!("{path}: {e}")))?;
    if bytes.is_empty() || bytes.len() > i32::MAX as usize {
        return Err(Error(format!("{path}: empty or too large for Cubism")));
    }
    Ok(bytes.into_owned())
}

fn resolve(base: &str, path: &str) -> Result<String, Error> {
    let path = path.replace('\\', "/");
    if path.is_empty() || path.starts_with('/') || path.contains(':') || path.contains('\0') {
        return Err(Error(format!("invalid model asset path: {path:?}")));
    }
    let mut parts: Vec<&str> = base.split('/').filter(|s| !s.is_empty()).collect();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(Error(format!("model asset escapes the asset root: {path}")));
                }
            }
            _ => parts.push(part),
        }
    }
    if parts.is_empty() {
        return Err(Error("model asset path names a directory".into()));
    }
    Ok(parts.join("/"))
}
