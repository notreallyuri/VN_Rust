use vn_script::{Catalog, LANG_DIR};

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
