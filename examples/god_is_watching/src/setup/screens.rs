use vn_engine::prelude::*;
use vn_engine::raylib::prelude::Color;
use vn_engine::screens::prelude::*;
use vn_engine::ui::layout::Anchor;
use vn_engine::ui::shape::Corners;

use super::{CASE_FILE, CREDITS, EVIDENCE, GALLERY, REPORT_DESK, SEARCH, screen};
use crate::screens as custom;
use crate::style;

pub fn screens(app: VnApp) -> VnApp {
    app.save_menu(|s| {
        s.background(style::background(style::ARCHIVE_BACKGROUND))
            .panel(style::frame)
            .panel_padding(32.0)
            .title_text(style::heading(40.0))
            .slot_size(540.0, 104.0)
            .slot_layout(|l| l.grid(2).anchor(Anchor::Center).spacing_xy(16.0, 12.0))
            .slot_hover_color(style::RAISED_HOVER)
            .slot_hover_border(1.0, style::BRASS)
            .thumbnail_color(Color::new(30, 24, 18, 200))
            .slot_title_text(TextStyle::new(FontRole::Menu, 19.0, style::PARCHMENT))
            .slot_summary_text(style::label(15.0))
            .delete_button(|b| b.size(72.0, 28.0).font_size(14.0))
            .back_button(|b| b.size(200.0, 44.0))
    })
    .settings(|s| {
        s.background(style::background(style::TITLE_BACKGROUND))
            .panel(style::frame)
            .title_text(style::heading(40.0))
            .label_text(TextStyle::new(FontRole::Menu, 21.0, style::TEXT))
            .value_button(|b| b.font_size(17.0))
            .focus_panel(|p| {
                p.color(style::RAISED.alpha(0.6))
                    .corners(Corners::bevel(style::CUT))
            })
            .value_text(style::label(18.0))
            .slider(style::slider)
            .sample_text("\"Your hot water, miss. Miss? Are you unwell?\"")
            .sample_sound("page_turn")
            .voice_row(false)
            .back_button(|b| b.size(200.0, 44.0))
    })
    .text_input(|t| {
        t.background(style::background(style::ARCHIVE_BACKGROUND))
            .panel(style::frame)
            .prompt_text(style::heading(32.0))
            .input_text(style::body(28.0))
            .input_box(|b| style::inset(b).border(1.0, style::BRASS))
            .hint("Type your name, then press Enter to sign")
            .hint_text(style::label(16.0))
    })
    .keybinds(|k| {
        k.title_text(style::heading(36.0))
            .section_text(style::section(18.0))
            .key_text(TextStyle::new(FontRole::Menu, 16.0, style::TEXT))
            .action_text(style::label(16.0))
            .header_text(style::label(13.0))
            .back_button(|b| b.size(180.0, 42.0))
            .section(
                KeySection::new("In the Archive")
                    .row("E, Esc", "B", "Close the evidence")
                    .row("Tab, Esc", "B", "Close the case file")
                    .row("E, Esc", "B", "Leave a room you are searching")
                    .row("Esc", "B", "Leave the report desk"),
            )
    })
    .log(|l| {
        l.title_text(style::heading(36.0))
            .speaker_text(TextStyle::new(FontRole::Speaker, 20.0, style::PARCHMENT))
            .line_text(style::body(19.0))
            .narration_text(style::body(19.0).color(style::MUTED))
            .choice_text(TextStyle::new(FontRole::Menu, 18.0, style::PARCHMENT))
            .choice_prefix("Decided: ")
            .empty_label("The page is blank.")
            .back_button(|b| b.size(180.0, 42.0))
    })
    .tooltips(|t| t.text(style::label(15.0).color(style::TEXT)))
    .toast(|t| {
        t.panel(|p| style::plate(p).border(1.0, style::BRASS))
            .text(TextStyle::new(FontRole::Menu, 17.0, style::PARCHMENT))
            .seconds(3.0)
    })
    .screen(screen(CREDITS), custom::CreditsScreen::new)
    .screen(screen(GALLERY), custom::GalleryScreen::new)
    .screen(screen(EVIDENCE), custom::EvidenceScreen::new)
    .screen(screen(SEARCH), custom::SearchScreen::new)
    .screen(screen(REPORT_DESK), custom::ReportDesk::new)
    .overlay(CASE_FILE, custom::CaseFileOverlay::new)
}
