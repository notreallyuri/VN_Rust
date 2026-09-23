use std::process::ExitCode;

use vn_engine::prelude::*;
use vn_engine::raylib::prelude::Color;
use vn_engine::screens::text_input::TextRequest;

const ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

const APPEARANCES: [&str; 3] = ["neutral", "cheerful", "annoyed"];

fn person(name: &str, color: (u8, u8, u8)) -> Character {
    Character::new(name).color(Color::new(color.0, color.1, color.2, 255))
}

#[command]
fn remember_name(ctx: &mut GameContext, variable: String) -> Option<ScreenState> {
    ctx.ask_text(
        TextRequest::new(variable, "What is your name?")
            .initial("Aoi")
            .max_len(20),
    )
}

#[cfg(feature = "live2d")]
fn kaede_model(app: VnApp) -> VnApp {
    use vn_live2d::{Appearance, Live2dCharacter};

    app.character_visual(
        "kaede",
        Live2dCharacter::new("characters/kaede/Hiyori.model3.json")
            .appearance("neutral", Appearance::new().motion("Idle", 0, true))
            .appearance("cheerful", Appearance::new().motion("Idle", 4, true))
            .appearance("annoyed", Appearance::new().motion("Idle", 7, true)),
    )
}

#[cfg(not(feature = "live2d"))]
fn kaede_model(app: VnApp) -> VnApp {
    app
}

fn main() -> ExitCode {
    let app = VnApp::new("Example VN")
        .assets(ASSETS)
        .character("me", person("", (220, 220, 220)))
        .character("unknown_1", person("???", (150, 170, 210)))
        .character(
            "kaede",
            person("Kaede", (225, 165, 180)).images(APPEARANCES),
        )
        .variable("player_name", VariableDef::string("Aoi"))
        .command(remember_name);

    match kaede_model(app).run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            println!("{e}");
            ExitCode::FAILURE
        }
    }
}
