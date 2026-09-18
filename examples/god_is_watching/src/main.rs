use std::process::ExitCode;

use vn_engine::raylib::prelude::*;
use vn_engine::{
    Action, FontRole, GameContext, MenuItem, PAUSE_OVERLAY, SAVE_OVERLAY, ScreenState, TextRequest,
    TextStyle, VnApp,
};

mod cast;
mod inventory;
mod screens;

use inventory::Inventory;

const ASSETS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");
const SAVES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/saves");

fn give_item(ctx: &mut GameContext, (item, count): (String, Option<u32>)) -> Option<ScreenState> {
    ctx.state
        .get_mut::<Inventory>()
        .add(&item, count.unwrap_or(1));
    None
}

fn ask_name(ctx: &mut GameContext, (variable,): (String,)) -> Option<ScreenState> {
    ctx.ask_text(TextRequest::new(variable, "What is your name?").max_len(20))
}

fn main() -> ExitCode {
    let credits = ScreenState::Custom("credits".into());
    let inventory = ScreenState::Custom("inventory".into());
    let muted_red = Color::new(80, 30, 30, 255);

    let app = cast::register(VnApp::new("God Is Watching"))
        .size(1280, 720)
        .assets(ASSETS_ROOT)
        .saves_dir(SAVES_DIR)
        .font(FontRole::Title, "NotoSerif-Regular.ttf")
        .font(FontRole::Dialogue, "NotoSerif-Regular.ttf")
        .state(Inventory::default())
        .command("give_item", give_item)
        .command("ask_name", ask_name)
        .start_screen(|s| s.prompt("PRESS ANY KEY TO START").footer("v0.1.0"))
        .main_menu(|m| {
            m.button_style(|b| b.size(260.0, 52.0))
                .button("New Game", Action::NewGame)
                .button("Continue", Action::Goto(ScreenState::Playing))
                .button("Load", Action::Goto(ScreenState::Load))
                .button("Settings", Action::Goto(ScreenState::Settings))
                .button("Credits", Action::Goto(credits.clone()))
                .item(MenuItem::new("Exit", Action::Quit).style(move |b| b.color(muted_red)))
        })
        .playing(|p| {
            p.dialogue_box(|b| b.height(180.0).roundness(0.08))
                .speaker_text(TextStyle::new(FontRole::Speaker, 26.0, Color::GOLD))
                .hud_button("Inventory", Action::Goto(inventory.clone()))
                .hud_button("Stats", Action::overlay("stats"))
                .hud_button("Save", Action::overlay(SAVE_OVERLAY))
                .hud_button("Menu", Action::overlay(PAUSE_OVERLAY))
        })
        .screen(credits, screens::CreditsScreen::new)
        .screen(inventory, screens::InventoryScreen::new)
        .overlay("stats", screens::StatsOverlay::new);

    match app.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("❌ {}", e);
            ExitCode::FAILURE
        }
    }
}
