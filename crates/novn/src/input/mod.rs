pub mod drag;
pub mod hit;
pub mod image_map;
pub mod navigation;
pub mod pad;
pub mod prompts;

pub mod prelude {
    pub use super::drag::{DragBoard, DragStyle, Draggable, DropTarget, Dropped};
    pub use super::hit::{Highlight, LabelAt, LabelStyle, Shape};
    pub use super::image_map::{Hotspot, HotspotPick, ImageMap, ImageMapStyle};
    pub use super::navigation::{Focus, InputDevice, NavInput, Navigation, NavigationConfig};
    pub use super::pad::{PadButton, PadFamily, PadLabels};
    pub use super::prompts::{Prompt, Prompts};
}
