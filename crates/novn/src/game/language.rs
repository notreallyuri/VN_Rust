use std::collections::BTreeSet;

use novn_script::{Catalog, LANG_DIR};

use crate::data::assets::Assets;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Language {
    pub code: Option<String>,
    pub label: String,
}

impl Language {
    pub fn source(label: impl Into<String>) -> Self {
        Self {
            code: None,
            label: label.into(),
        }
    }

    pub fn new(code: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            code: Some(code.into()),
            label: label.into(),
        }
    }

    pub fn is(&self, code: Option<&str>) -> bool {
        self.code.as_deref() == code
    }
}

pub fn catalog_file(code: &str) -> String {
    format!("{}/{}.json", LANG_DIR, code)
}

pub fn load_catalog(assets: &Assets, code: &str) -> Result<Catalog, String> {
    let file = catalog_file(code);
    let json = assets
        .read_to_string(&file)
        .map_err(|e| format!("{}: {}", assets.describe(&file), e))?;
    Catalog::from_json(&json).map_err(|e| format!("{}: {}", assets.describe(&file), e))
}

pub fn charset(program: &novn_script::Program, catalogs: &[Catalog]) -> BTreeSet<char> {
    let mut chars = BTreeSet::new();
    let mut add = |text: &str| chars.extend(text.chars());

    for instruction in &program.instructions {
        match instruction {
            novn_script::Instruction::Say { text, .. } => add(text),
            novn_script::Instruction::Choice { options, .. } => {
                for arm in options {
                    add(&arm.text);
                    if let Some(reason) = arm.gate.as_ref().and_then(|gate| gate.reason.as_deref())
                    {
                        add(reason);
                    }
                }
            }
            _ => {}
        }
    }

    for catalog in catalogs {
        for entry in catalog.entries() {
            add(&entry.source);
            add(&entry.text);
        }
    }

    chars
}
