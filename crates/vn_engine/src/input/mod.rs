pub mod drag;
pub mod hit;
pub mod image_map;
pub mod navigation;

pub mod prelude {
    pub use super::drag::{DragBoard, DragStyle, Draggable, DropTarget, Dropped};
    pub use super::hit::{Highlight, LabelAt, LabelStyle, Shape};
    pub use super::image_map::{Hotspot, HotspotPick, ImageMap, ImageMapStyle};
    pub use super::navigation::{Focus, NavInput, Navigation, NavigationConfig};
}
