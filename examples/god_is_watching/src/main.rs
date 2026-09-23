use std::process::ExitCode;

use vn_engine::prelude::*;

mod cast;
mod commands;
mod desk;
mod evidence;
mod gallery;
mod journal;
mod screens;
mod setup;
mod style;

fn main() -> ExitCode {
    let app = VnApp::new("God Is Watching")
        .theme(style::theme)
        .with(setup::window)
        .with(setup::registries)
        .with(setup::menus)
        .with(setup::playing)
        .with(setup::screens);

    match app.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("❌ {}", e);
            ExitCode::FAILURE
        }
    }
}
