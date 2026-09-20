pub mod action;
pub mod app;
pub mod context;
pub mod data;
pub mod frame;
pub mod game;
pub mod input;
pub mod overlay;
pub mod screen;
pub mod screen_manager;
pub mod screens;
pub mod ui;

pub mod prelude {
    pub use crate::action::Action;
    pub use crate::app::{AppError, VnApp};
    pub use crate::context::{DrawContext, GameContext, GameView};
    pub use crate::game::characters::Character;
    pub use crate::overlay::{Overlay, OverlayAction, OverlayRequest};
    pub use crate::screen::{Screen, ScreenState};
    pub use crate::ui::TextStyle;
    pub use crate::ui::fonts::FontRole;
    pub use crate::ui::shape::PanelStyle;
    pub use vn_script::{Value, VariableDef};
}

pub use data::{assets, resources, rollback, saves, session, settings, state};
pub use frame::{effects, post, scenery, screen_transition, stage, target, viewport};
pub use game::{audio, characters, commands, hooks, hot_reload, language, script_errors};
pub use input::{drag, hit, image_map, navigation};
pub use ui::{button, ease, fonts, layout, scroll, shape, styled, toast, tooltip};

pub use action::*;
pub use app::*;
pub use assets::{Assets, EmbeddedFile};
pub use audio::*;
pub use button::{
    Border, Button, ButtonIcon, ButtonImage, ButtonLook, ButtonStyle, IconSide, Shadow, Slice,
    StyleOverride, TextAlign, TextOverflow, Transform,
};
pub use characters::*;
pub use commands::*;
pub use context::*;
pub use drag::{DragBoard, DragStyle, Draggable, DropTarget, Dropped};
pub use ease::{Easing, Tween, lerp};
pub use effects::{ScreenEffects, ScreenEffectsConfig};
pub use fonts::*;
pub use hit::{Highlight, LabelAt, LabelStyle, Shape};
pub use hooks::*;
pub use hot_reload::*;
pub use image_map::{Hotspot, HotspotPick, ImageMap, ImageMapStyle};
pub use language::{Language, load_catalog};
pub use layout::{Align, Anchor, Arrangement, Layout};
pub use navigation::*;
pub use overlay::*;
pub use post::{Pass, PostChain};
pub use resources::*;
pub use rollback::*;
pub use saves::{
    AUTO_SLOT, LoadReport, LoadWarning, MigrationFn, Migrations, QUICK_SLOT, SAVE_FORMAT_VERSION,
    SaveError, SaveFile, SaveMigration, Saves, SlotInfo, THUMBNAIL_WIDTH, default_saves_dir, slug,
};
pub use scenery::{Letterbox, Motion, Scenery, Vignette};
pub use screen::*;
pub use screen_manager::*;
pub use screen_transition::{ScreenTransition, ScreenTransitionConfig, ScreenTransitionKind};
pub use screens::*;
pub use script_errors::*;
pub use scroll::{Scroll, ScrollStyle};
pub use session::*;
pub use settings::*;
pub use shape::{Corner, CornerShape, Corners, Gradient, GradientDirection, PanelStyle};
pub use stage::*;
pub use state::*;
pub use styled::{StyledLine, StyledText};
pub use toast::*;
pub use tooltip::*;
pub use ui::{Background, SliderStyle, TextStyle};
pub use viewport::Viewport;

pub use raylib;
pub use serde;
pub use serde_json;
pub use vn_script as script;
pub use vn_script::{Value, VariableDef};
