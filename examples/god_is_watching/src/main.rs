use std::process::ExitCode;

use vn_engine::{
    Action, Align, FontRole, GameContext, HudButton, MenuItem, PAUSE_OVERLAY, ScreenState,
    TextRequest, TextStyle, VnApp,
};

mod cast;
mod evidence;
mod journal;
mod screens;
mod style;

use evidence::Evidence;
use journal::{Journal, achievement_name, chapter_title};

const ASSETS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
const CASE_FILE: &str = "case_file";

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

fn unlock(ctx: &mut GameContext, (key,): (String,)) -> Option<ScreenState> {
    if ctx.state.get_mut::<Journal>().unlock(&key) {
        ctx.notify(achievement_name(&key));
    }
    None
}

fn main() -> ExitCode {
    let credits = ScreenState::Custom("credits".into());
    let evidence_screen = ScreenState::Custom("evidence".into());
    let heading = |size| TextStyle::new(FontRole::Title, size, style::PARCHMENT);

    let app = cast::register(VnApp::new("God Is Watching"))
        .size(1280, 720)
        .assets(ASSETS_ROOT)
        .clear_color(style::INK)
        .audio(|a| a.menu_music("title").fade_seconds(1.5))
        .font(FontRole::Title, "NotoSerif-Regular.ttf")
        .font(FontRole::Dialogue, "NotoSerif-Regular.ttf")
        .font(FontRole::Speaker, "NotoSerif-Regular.ttf")
        .state(Evidence::default())
        .state(Journal::default())
        .command("give_item", give_item)
        .command("ask_name", ask_name)
        .command("note", note)
        .command("unlock", unlock)
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
                .prompt("PRESS ANY KEY")
                .prompt_text(TextStyle::new(FontRole::Menu, 22.0, style::PARCHMENT))
                .footer("v0.2 · made with VN_Rust")
                .footer_text(style::label(16.0))
        })
        .main_menu(|m| {
            m.background(style::background(style::TITLE_BACKGROUND))
                .title_text(heading(72.0))
                .title_y(0.2)
                .button_style(|b| style::menu_button(b.size(260.0, 52.0)))
                .layout(|l| {
                    l.rows_of([1, 2, 2, 1])
                        .align(Align::Stretch)
                        .spacing_xy(24.0, 16.0)
                })
                .button("New Game", Action::NewGame)
                .item(
                    MenuItem::new("Continue", Action::Continue)
                        .tooltip("Back to where you left off, or your most recent save"),
                )
                .button("Load", Action::Goto(ScreenState::Load))
                .button("Settings", Action::Goto(ScreenState::Settings))
                .button("Credits", Action::Goto(credits.clone()))
                .item(
                    MenuItem::new("Exit", Action::confirm("Leave the Archive?", Action::Quit))
                        .style(style::danger_button),
                )
        })
        .playing(|p| {
            p.dialogue_box(|b| b.height(190.0).color(style::PANEL).roundness(0.06))
                .speaker_text(TextStyle::new(FontRole::Speaker, 26.0, style::PARCHMENT))
                .dialogue_text(style::body(25.0))
                .choice_button(|b| style::choice_button(b.size(780.0, 60.0)).font_size(21.0))
                .end_title_text(heading(60.0))
                .end_hint("Click to see the credits")
                .end_hint_text(style::label(20.0))
                .after_end(credits.clone())
                .hud_button_style(|b| style::hud_button(b.size(136.0, 38.0)).font_size(17.0))
                .hud_item(
                    HudButton::new("Evidence", Action::Goto(evidence_screen.clone()))
                        .style(|b| b.icon(style::icon("icon_evidence")))
                        .tooltip("What you have filed so far"),
                )
                .hud_item(
                    HudButton::new("Case file", Action::overlay(CASE_FILE))
                        .style(|b| b.icon(style::icon("icon_case_file")))
                        .tooltip("Your notes, your decisions and what the Archive knows"),
                )
                .hud_item(
                    HudButton::new("Menu", Action::overlay(PAUSE_OVERLAY))
                        .style(|b| b.icon(style::icon("icon_menu")))
                        .tooltip("Pause (Esc)"),
                )
        })
        .pause_menu(|p| {
            p.title_text(heading(40.0))
                .button_style(|b| style::button(b.size(240.0, 46.0)))
                .layout(|l| l.grid(2).spacing_xy(14.0, 12.0))
                .panel_color(style::PANEL)
                .backdrop(style::BACKDROP)
        })
        .confirm_dialog(|c| {
            c.message_text(style::body(22.0))
                .panel_color(style::PANEL)
                .backdrop(style::BACKDROP)
                .cancel_button(style::button)
                .confirm_button(style::danger_button)
        })
        .save_menu(|s| {
            s.background(style::background(style::ARCHIVE_BACKGROUND))
                .backdrop(style::BACKDROP)
                .title_text(heading(44.0))
                .slot_size(560.0, 84.0)
                .slot_layout(|l| l.grid(2).spacing_xy(20.0, 14.0))
                .slot_color(style::PANEL)
                .slot_title_text(TextStyle::new(FontRole::Menu, 20.0, style::PARCHMENT))
                .slot_summary_text(style::label(16.0))
                .back_button(|b| style::button(b.size(200.0, 48.0)))
        })
        .settings(|s| {
            s.background(style::background(style::TITLE_BACKGROUND))
                .backdrop(style::BACKDROP)
                .title_text(heading(44.0))
                .label_text(TextStyle::new(FontRole::Menu, 24.0, style::TEXT))
                .value_button(style::button)
                .value_text(style::label(18.0))
                .slider(style::slider)
                .sample_text("\"Your hot water, miss. Miss? Are you unwell?\"")
                .sample_sound("page_turn")
                .back_button(|b| style::button(b.size(200.0, 48.0)))
        })
        .text_input(|t| {
            t.background(style::background(style::ARCHIVE_BACKGROUND))
                .prompt_text(heading(34.0))
                .input_text(style::body(30.0))
                .box_color(style::PANEL)
                .box_border(style::PARCHMENT)
                .hint("Type your name, then press Enter to sign")
                .hint_text(style::label(16.0))
        })
        .tooltips(|t| {
            t.background(style::PANEL)
                .border(Some(style::MUTED))
                .text(TextStyle::new(FontRole::Menu, 16.0, style::TEXT))
        })
        .toast(|t| {
            t.background(style::PANEL)
                .text(TextStyle::new(FontRole::Menu, 18.0, style::PARCHMENT))
                .seconds(3.0)
        })
        .screen(credits, screens::CreditsScreen::new)
        .screen(evidence_screen, screens::EvidenceScreen::new)
        .overlay(CASE_FILE, screens::CaseFileOverlay::new);

    match app.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("❌ {}", e);
            ExitCode::FAILURE
        }
    }
}
