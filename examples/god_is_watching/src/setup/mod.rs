use vn_engine::prelude::*;

mod menus;
mod playing;
mod registries;
mod screens;
mod window;

pub use menus::menus;
pub use playing::playing;
pub use registries::registries;
pub use screens::screens;
pub use window::window;

pub const ASSETS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
pub const SUBTITLE: &str = "AN ACCOUNT FROM THE ARCHIVE OF THE HOUSE  ·  1903";

pub const CASE_FILE: &str = "case_file";
pub const CREDITS: &str = "credits";
pub const EVIDENCE: &str = "evidence";
pub const GALLERY: &str = "gallery";
pub const REPORT_DESK: &str = "report_desk";
pub const SEARCH: &str = "search";

pub fn screen(name: &str) -> ScreenState {
    ScreenState::Custom(name.to_string())
}
