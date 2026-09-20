use std::process::ExitCode;

use vn_engine::prelude::*;
use vn_engine::raylib::prelude::Color;
use vn_engine::screens::prelude::*;
use vn_engine::ui::layout::Anchor;
use vn_engine::ui::shape::Corners;

mod cast;
mod desk;
mod evidence;
mod journal;
mod screens;
mod style;

use desk::Desk;
use evidence::Evidence;
use journal::{Journal, achievement_name, chapter_title};

const ASSETS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
const CASE_FILE: &str = "case_file";
const SEARCH: &str = "search";
const REPORT_DESK: &str = "report_desk";
const SUBTITLE: &str = "AN ACCOUNT FROM THE ARCHIVE OF THE HOUSE  ·  1903";

fn give_item(ctx: &mut GameContext, (item, count): (String, Option<u32>)) -> Option<ScreenState> {
    ctx.state
        .get_mut::<Evidence>()
        .add(&item, count.unwrap_or(1));
    ctx.notify(format!("Filed: {}", evidence::describe(&item).0));
    None
}

fn ask_name(ctx: &mut GameContext, (variable,): (String,)) -> Option<ScreenState> {
    ctx.ask_text(
        TextRequest::new(variable, "Write your name in the ledger")
            .initial("Archivist")
            .max_len(20),
    )
}

fn note(ctx: &mut GameContext, (key,): (String,)) -> Option<ScreenState> {
    if ctx.state.get_mut::<Journal>().note(&key) {
        ctx.notify("Added to the case file");
    }
    None
}

fn search(ctx: &mut GameContext, (room,): (String,)) -> Option<ScreenState> {
    ctx.state.get_mut::<Desk>().search(&room);
    Some(ScreenState::Custom(SEARCH.to_string()))
}

fn assemble(_ctx: &mut GameContext, _: ()) -> Option<ScreenState> {
    Some(ScreenState::Custom(REPORT_DESK.to_string()))
}

fn unlock(ctx: &mut GameContext, (key,): (String,)) -> Option<ScreenState> {
    if ctx.state.get_mut::<Journal>().unlock(&key) {
        ctx.notify(achievement_name(&key));
    }
    None
}

fn link(label: &'static str, action: Action) -> MenuItem {
    MenuItem::new(label, action).style(move |b| b.size(style::link_width(label), 36.0))
}

fn main() -> ExitCode {
    let credits = ScreenState::Custom("credits".into());
    let evidence_screen = ScreenState::Custom("evidence".into());
    let heading = |size| TextStyle::new(FontRole::Title, size, style::PARCHMENT);

    let app = cast::register(VnApp::new("God Is Watching"))
        .size(1280, 720)
        .render_scale(1.5)
        .assets(ASSETS_ROOT)
        .embedded_assets(vn_engine::embedded_assets!())
        .clear_color(style::INK)
        .source_language("English")
        .language("pt-BR", "Português (BR)")
        .audio(|a| a.menu_music("title").fade_seconds(1.5))
        .font(FontRole::Title, "NotoSerif-Regular.ttf")
        .font(FontRole::Dialogue, "NotoSerif-Regular.ttf")
        .font(FontRole::Speaker, "NotoSerif-Regular.ttf")
        .state(Evidence::default())
        .state(Journal::default())
        .state(Desk::default())
        .command("give_item", give_item)
        .command("ask_name", ask_name)
        .command("note", note)
        .command("unlock", unlock)
        .command("search", search)
        .command("assemble", assemble)
        .on_scene_enter(|ctx, scene| {
            if let Some(title) = chapter_title(scene) {
                ctx.notify(title);
            }
            None
        })
        .on_choice(|ctx, _, text| {
            ctx.state.get_mut::<Journal>().decide(text);
            None
        })
        .rollback(|r| r.block_command("unlock"))
        .start_screen(|s| {
            s.background(style::background(style::TITLE_BACKGROUND))
                .scenery(|s| style::scenery(s).letterbox(|l| l.slide_in(1.6)))
                .title("GOD IS WATCHING")
                .title_text(style::title_card())
                .title_y(style::TITLE_CARD_Y)
                .subtitle(SUBTITLE)
                .subtitle_text(style::title_card_subtitle())
                .prompt("PRESS ANY KEY")
                .prompt_text(style::section(14.0).spacing(6.0))
                .prompt_in_bar(true)
                .prompt_pulse(2.6)
                .footer("v0.3 · made with VN_Rust")
                .footer_text(style::label(12.0))
        })
        .main_menu(|m| {
            m.background(style::background(style::TITLE_BACKGROUND))
                .scenery(style::scenery)
                .intro(1.4)
                .title("GOD IS WATCHING")
                .title_text(style::title_card())
                .title_y(style::TITLE_CARD_Y)
                .subtitle(SUBTITLE)
                .subtitle_text(style::title_card_subtitle())
                .buttons_in_bar(true)
                .button_style(|b| style::menu_link(b.size(132.0, 36.0)))
                .layout(|l| l.row().anchor(Anchor::Center).spacing(34.0))
                .separator(7.0, |p| p.color(style::BRASS_DIM))
                .item(link("NEW GAME", Action::NewGame))
                .item(
                    link("CONTINUE", Action::Continue)
                        .tooltip("Back to where you left off, or your most recent save"),
                )
                .item(link("LOAD", Action::Goto(ScreenState::Load)))
                .item(link("SETTINGS", Action::Goto(ScreenState::Settings)))
                .item(link("CREDITS", Action::Goto(credits.clone())))
                .item(
                    MenuItem::new("EXIT", Action::confirm("Leave the Archive?", Action::Quit))
                        .style(|b| {
                            style::menu_link_danger(b.size(style::link_width("EXIT"), 36.0))
                        }),
                )
        })
        .playing(|p| {
            p.dialogue_box(|b| {
                b.height(150.0)
                    .margin(56.0)
                    .bottom(52.0)
                    .max_width(1120.0)
                    .padding(30.0)
                    .panel(style::frame)
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
            .indicator(|i| style::plate(i).border(1.0, style::BRASS))
            .indicator_text(style::section(15.0))
            .end_title_text(heading(60.0))
            .end_hint("Click to see the credits")
            .end_hint_text(style::label(20.0))
            .after_end(credits.clone())
            .hud_button_style(|b| style::hud_button(b.size(112.0, 30.0)))
            .hud_margin(12.0)
            .hud_layout(|l| l.row().anchor(Anchor::Bottom).spacing(6.0))
            .hud_group("top", |l| l.row().anchor(Anchor::TopRight).spacing(8.0))
            .hud_item(
                HudButton::new("Evidence", Action::Goto(evidence_screen.clone()))
                    .group("top")
                    .style(|b| {
                        style::hud_chip(b.size(124.0, 32.0)).icon(style::icon("icon_evidence"))
                    })
                    .tooltip("What you have filed so far"),
            )
            .hud_item(
                HudButton::new("Case file", Action::overlay(CASE_FILE))
                    .group("top")
                    .style(|b| {
                        style::hud_chip(b.size(124.0, 32.0)).icon(style::icon("icon_case_file"))
                    })
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
        .pause_menu(|p| {
            p.title_text(heading(38.0))
                .button_style(|b| style::button(b.size(230.0, 44.0)))
                .layout(|l| l.grid(2).spacing_xy(12.0, 10.0))
                .panel(style::frame)
                .backdrop(style::BACKDROP)
        })
        .confirm_dialog(|c| {
            c.message_text(style::body(22.0))
                .panel(style::frame)
                .backdrop(style::BACKDROP)
                .cancel_button(style::button)
                .confirm_button(style::danger_button)
        })
        .save_menu(|s| {
            s.background(style::background(style::ARCHIVE_BACKGROUND))
                .backdrop(style::BACKDROP)
                .panel(style::frame)
                .panel_padding(32.0)
                .title_text(heading(40.0))
                .slot_size(540.0, 104.0)
                .slot_layout(|l| l.grid(2).anchor(Anchor::Center).spacing_xy(16.0, 12.0))
                .slot_panel(style::inset)
                .slot_hover_color(style::RAISED_HOVER)
                .slot_hover_border(1.0, style::BRASS)
                .thumbnail_color(Color::new(30, 24, 18, 200))
                .slot_title_text(TextStyle::new(FontRole::Menu, 19.0, style::PARCHMENT))
                .slot_summary_text(style::label(15.0))
                .delete_button(|b| style::danger_button(b.size(72.0, 28.0)).font_size(14.0))
                .back_button(|b| style::button(b.size(200.0, 44.0)))
        })
        .settings(|s| {
            s.background(style::background(style::TITLE_BACKGROUND))
                .backdrop(style::BACKDROP)
                .panel(style::frame)
                .title_text(heading(40.0))
                .label_text(TextStyle::new(FontRole::Menu, 21.0, style::TEXT))
                .value_button(|b| style::button(b).font_size(17.0))
                .focus_panel(|p| {
                    p.color(style::RAISED.alpha(0.6))
                        .corners(Corners::bevel(style::CUT))
                })
                .sample_box(style::inset)
                .value_text(style::label(18.0))
                .slider(style::slider)
                .sample_text("\"Your hot water, miss. Miss? Are you unwell?\"")
                .sample_sound("page_turn")
                .voice_row(false)
                .back_button(|b| style::button(b.size(200.0, 44.0)))
        })
        .text_input(|t| {
            t.background(style::background(style::ARCHIVE_BACKGROUND))
                .panel(style::frame)
                .prompt_text(heading(32.0))
                .input_text(style::body(28.0))
                .input_box(|b| style::inset(b).border(1.0, style::BRASS))
                .hint("Type your name, then press Enter to sign")
                .hint_text(style::label(16.0))
        })
        .keybinds(|k| {
            k.title_text(heading(36.0))
                .panel(style::frame)
                .backdrop(style::BACKDROP)
                .section_text(style::section(18.0))
                .key_text(TextStyle::new(FontRole::Menu, 16.0, style::TEXT))
                .action_text(style::label(16.0))
                .header_text(style::label(13.0))
                .back_button(|b| style::button(b.size(180.0, 42.0)))
                .section(
                    KeySection::new("In the Archive")
                        .row("E, Esc", "B", "Close the evidence")
                        .row("Tab, Esc", "B", "Close the case file")
                        .row("E, Esc", "B", "Leave a room you are searching")
                        .row("Esc", "B", "Leave the report desk"),
                )
        })
        .log(|l| {
            l.title_text(heading(36.0))
                .panel(style::frame)
                .backdrop(style::BACKDROP)
                .speaker_text(TextStyle::new(FontRole::Speaker, 20.0, style::PARCHMENT))
                .line_text(style::body(19.0))
                .narration_text(style::body(19.0).color(style::MUTED))
                .choice_text(TextStyle::new(FontRole::Menu, 18.0, style::PARCHMENT))
                .choice_prefix("Decided: ")
                .empty_label("The page is blank.")
                .back_button(|b| style::button(b.size(180.0, 42.0)))
        })
        .tooltips(|t| {
            t.panel(style::plate)
                .text(TextStyle::new(FontRole::Menu, 15.0, style::TEXT))
        })
        .toast(|t| {
            t.panel(|p| style::plate(p).border(1.0, style::BRASS))
                .text(TextStyle::new(FontRole::Menu, 17.0, style::PARCHMENT))
                .seconds(3.0)
        })
        .screen(credits, screens::CreditsScreen::new)
        .screen(evidence_screen, screens::EvidenceScreen::new)
        .screen(
            ScreenState::Custom(SEARCH.to_string()),
            screens::SearchScreen::new,
        )
        .screen(
            ScreenState::Custom(REPORT_DESK.to_string()),
            screens::ReportDesk::new,
        )
        .overlay(CASE_FILE, screens::CaseFileOverlay::new);

    match app.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("❌ {}", e);
            ExitCode::FAILURE
        }
    }
}
