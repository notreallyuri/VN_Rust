use novn::prelude::*;
use novn::screens::text_input::TextRequest;

use crate::desk::{Desk, Room};
use crate::evidence::{Evidence, EvidenceItem};
use crate::journal::{Achievement, Achievements, CaseLedger, Ending, Journal, Note};
use crate::setup::{REPORT_DESK, SEARCH, screen};

#[command]
fn give_item(ctx: &mut GameContext, item: EvidenceItem, count: Option<u32>) -> Option<ScreenState> {
    ctx.state
        .get_mut::<Evidence>()
        .add(item, count.unwrap_or(1));
    ctx.notify(format!("Filed: {}", item.describe().0));
    None
}

#[command]
fn ask_name(ctx: &mut GameContext, variable: String) -> Option<ScreenState> {
    ctx.ask_text(
        TextRequest::new(variable, "Write your name in the ledger")
            .initial("Archivist")
            .max_len(20),
    )
}

#[command]
fn note(ctx: &mut GameContext, key: Note) -> Option<ScreenState> {
    if ctx.state.get_mut::<Journal>().note(key) {
        ctx.notify("Added to the case file");
    }
    None
}

#[command]
fn search(ctx: &mut GameContext, room: Room) -> Option<ScreenState> {
    ctx.state.get_mut::<Desk>().search(room);
    Some(screen(SEARCH))
}

#[command]
fn assemble(_ctx: &mut GameContext) -> Option<ScreenState> {
    Some(screen(REPORT_DESK))
}

#[command]
fn unlock(ctx: &mut GameContext, key: Achievement) -> Option<ScreenState> {
    if ctx.persistent.get_mut::<Achievements>().unlock(key) {
        ctx.notify(key.name());
    }
    None
}

#[command]
fn close_case(ctx: &mut GameContext, ending: Ending) -> Option<ScreenState> {
    ctx.persistent.get_mut::<CaseLedger>().close(ending);
    None
}

#[command]
fn remember_cases(ctx: &mut GameContext) -> Option<ScreenState> {
    let closed = ctx.persistent.get::<CaseLedger>().closed() as i32;
    if let Err(e) = ctx.story.set_variable("cases_closed", Value::Int(closed)) {
        eprintln!("⚠️ remember_cases: {}", e);
    }
    None
}

pub fn register(app: VnApp) -> VnApp {
    app.command(give_item)
        .command(ask_name)
        .command(note)
        .command(unlock)
        .command(close_case)
        .command(remember_cases)
        .command(search)
        .command(assemble)
}
