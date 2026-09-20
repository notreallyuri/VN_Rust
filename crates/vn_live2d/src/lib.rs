//! Optional Cubism Native integration. Asset inspection works without the SDK.
//! Enable `native` for the experimental Linux/OpenGL viewer and native runtime.

mod assets;
mod character;
pub use assets::{Expression, ModelAssets, ModelSettings, Motion};
pub use character::{Appearance, Live2dCharacter};

#[cfg(feature = "engine")]
mod engine;

#[cfg(feature = "native")]
mod native;
#[cfg(feature = "native")]
pub use native::{Cubism, Model, with_cubism};

#[derive(Debug)]
pub struct Error(pub(crate) String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}
