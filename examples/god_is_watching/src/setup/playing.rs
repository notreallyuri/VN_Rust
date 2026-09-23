use vn_engine::prelude::*;
use vn_engine::screens::prelude::*;
use vn_engine::ui::layout::Anchor;

use super::{CASE_FILE, CREDITS, EVIDENCE, screen};
use crate::style;

pub fn playing(app: VnApp) -> VnApp {
    app.playing(|p| {
        p.dialogue_box(|b| {
            b.height(150.0)
                .margin(56.0)
                .bottom(52.0)
                .max_width(1120.0)
                .padding(30.0)
                .name_plate(|n| {
                    n.panel(style::plate)
                        .padding(22.0, 7.0)
                        .indent(34.0)
                        .overlap(16.0)
                        .min_width(150.0)
                })
        })
        .speaker_text(TextStyle::new(FontRole::Speaker, 22.0, style::PARCHMENT))
        .dialogue_text(style::body(24.0))
        .choice_button(|b| style::choice_button(b.size(760.0, 58.0)).font_size(21.0))
        .choice_layout(|l| l.spacing(12.0))
        .choice_preview(|v| {
            v.size(420.0, 236.0)
                .anchor(Anchor::TopRight)
                .margin(56.0)
                .padding(8.0)
        })
        .nvl(|n| {
            n.margin(72.0)
                .padding(44.0)
                .max_width(Some(1040.0))
                .entry_spacing(16.0)
        })
        .indicator(|i| i.border(1.0, style::BRASS))
        .indicator_text(style::section(15.0))
        .end_title_text(style::heading(60.0))
        .end_hint("{advance} to see the credits")
        .end_hint_text(style::label(20.0))
        .after_end(screen(CREDITS))
        .hud_button_style(|b| style::hud_button(b.size(112.0, 30.0)))
        .hud_margin(12.0)
        .hud_layout(|l| l.row().anchor(Anchor::Bottom).spacing(6.0))
        .hud_group("top", |l| l.row().anchor(Anchor::TopRight).spacing(8.0))
        .hud_item(
            HudButton::new("Evidence", Action::Goto(screen(EVIDENCE)))
                .group("top")
                .style(|b| style::hud_chip(b.size(124.0, 32.0)).icon(style::icon("icon_evidence")))
                .tooltip("What you have filed so far"),
        )
        .hud_item(
            HudButton::new("Case file", Action::overlay(CASE_FILE))
                .group("top")
                .style(|b| style::hud_chip(b.size(124.0, 32.0)).icon(style::icon("icon_case_file")))
                .tooltip("Your notes, your decisions and what the Archive knows"),
        )
        .hud_item(
            HudButton::new("Log", Action::overlay(LOG_OVERLAY))
                .style(|b| b.icon(style::icon("icon_log")))
                .tooltip("Everything read and decided so far (L)"),
        )
        .hud_item(
            HudButton::new("Auto", Action::ToggleAuto)
                .style(|b| b.icon(style::icon("icon_auto")))
                .tooltip("Turn the pages on their own (A). Tab or Ctrl skips read pages"),
        )
        .hud_item(
            HudButton::new("Menu", Action::overlay(PAUSE_OVERLAY))
                .style(|b| b.icon(style::icon("icon_menu")))
                .tooltip("Pause (Esc)"),
        )
    })
}
