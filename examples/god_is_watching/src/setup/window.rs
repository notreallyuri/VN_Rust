use novn::prelude::*;
use novn::ui::cursor::{CursorKind, CursorStyle};

use super::ASSETS_ROOT;
use crate::style;

pub fn window(app: VnApp) -> VnApp {
    app.size(1280, 720)
        .render_scale(1.5)
        .assets(ASSETS_ROOT)
        .embedded_assets(novn::embedded_assets!())
        .clear_color(style::INK)
        .cursor(
            CursorStyle::new("cursor.png")
                .hand("cursor_hand.png")
                .picture(CursorKind::Text, "cursor_text.png")
                .picture(CursorKind::Grab, "cursor_grab.png")
                .picture(CursorKind::Grabbing, "cursor_grabbing.png")
                .picture(CursorKind::NotAllowed, "cursor_no.png")
                .size(28.0)
                .hotspot(0.128, 0.094)
                .hotspot_for(CursorKind::Text, 0.5, 0.5)
                .hotspot_for(CursorKind::Grab, 0.5, 0.5)
                .hotspot_for(CursorKind::Grabbing, 0.5, 0.5)
                .hotspot_for(CursorKind::NotAllowed, 6.0 / 61.0, 6.0 / 64.0),
        )
        .source_language("English")
        .language("pt-BR", "Português (BR)")
        .audio(|a| a.menu_music("title").fade_seconds(1.5))
        .font(FontRole::Title, "NotoSerif-Regular.ttf")
        .font(FontRole::Dialogue, "NotoSerif-Regular.ttf")
        .font(FontRole::Speaker, "NotoSerif-Regular.ttf")
}
