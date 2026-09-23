use vn_engine::prelude::*;

use crate::cast;
use crate::commands;
use crate::desk::Desk;
use crate::evidence::Evidence;
use crate::journal::{Achievements, CaseLedger, Journal, chapter_title};

pub fn registries(app: VnApp) -> VnApp {
    commands::register(cast::register(app))
        .state(Evidence::default())
        .state(Journal::default())
        .persistent(Achievements::default())
        .persistent(CaseLedger::default())
        .state(Desk::default())
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
}
