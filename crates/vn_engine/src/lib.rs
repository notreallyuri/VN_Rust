pub mod action;
pub mod app;
pub mod context;
pub mod data;
pub mod frame;
pub mod game;
pub mod input;
pub mod overlay;
pub mod request;
pub mod screen;
pub mod screen_manager;
pub mod screens;
pub mod ui;
#[cfg(any(feature = "video-portable", feature = "video-ffmpeg"))]
pub mod video;

pub mod prelude {
    pub use crate::action::Action;
    pub use crate::app::{AppError, VnApp};
    pub use crate::context::{DrawContext, GameContext, GameView};
    pub use crate::game::characters::Character;
    pub use crate::game::commands::Command;
    pub use crate::overlay::{Overlay, OverlayAction, OverlayRequest};
    pub use crate::request::{Request, Requests, Resolved};
    pub use crate::screen::{Screen, ScreenState};
    pub use crate::ui::TextStyle;
    pub use crate::ui::fonts::FontRole;
    pub use crate::ui::shape::PanelStyle;
    pub use vn_macros::{StoryWord, command};
    pub use vn_script::{Value, VariableDef};
}

pub use raylib;
pub use serde;
pub use serde_json;
pub use vn_macros::{StoryWord, command};
pub use vn_script as script;
