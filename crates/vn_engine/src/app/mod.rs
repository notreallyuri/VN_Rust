mod builder;
mod check;
mod error;
mod run;
mod screens;

pub use builder::*;
pub(crate) use check::missing_art;
pub use error::*;
pub use screens::*;

use crate::overlay::Overlay;
use crate::screen::Screen;

pub(crate) type ScreenBuilder = Box<dyn Fn() -> Box<dyn Screen>>;
pub(crate) type OverlayBuilder = Box<dyn Fn() -> Box<dyn Overlay>>;
