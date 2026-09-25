pub mod audio;
pub mod characters;
pub mod commands;
pub mod hooks;
pub mod hot_reload;
pub mod language;
#[cfg(feature = "character-visuals")]
pub mod puppet;
pub mod script_errors;
#[cfg(feature = "character-visuals")]
pub mod visuals;

pub mod prelude {
    pub use super::audio::{Audio, AudioConfig, Fade};
    pub use super::characters::{Character, Characters};
    pub use super::commands::{Arg, Commands, FromArg, FromArgs};
    pub use super::hooks::Hooks;
    pub use super::language::{Language, load_catalog};
    pub use super::script_errors::ScriptErrors;
}
