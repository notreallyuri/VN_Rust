use crate::context::GameContext;
use crate::data::saves::{AUTO_SLOT, QUICK_SLOT, SaveFile, SavePoint, time_ago};

pub(super) fn save_to(ctx: &mut GameContext, slot: &str) {
    match ctx.save(slot) {
        Ok(()) => {
            let message = ctx.message("Saved to {slot}", &[("slot", &slot_label(ctx, slot))]);
            ctx.notify(message);
        }
        Err(e) => {
            eprintln!("⚠️ Save failed ({}): {}", slot, e);
            let message = ctx.message("Save failed: {reason}", &[("reason", &e.player_message())]);
            ctx.notify_error(message);
        }
    }
}

pub(super) fn load_from(ctx: &mut GameContext, slot: &str) -> bool {
    match ctx.load(slot) {
        Ok(_) => {
            let message = ctx.message("Loaded {slot}", &[("slot", &slot_label(ctx, slot))]);
            ctx.notify(message);
            true
        }
        Err(e) => {
            eprintln!("⚠️ Load failed ({}): {}", slot, e);
            let message = ctx.message("Load failed: {reason}", &[("reason", &e.player_message())]);
            ctx.notify_error(message);
            false
        }
    }
}

pub(super) fn delete_slot(ctx: &mut GameContext, slot: &str) {
    match ctx.saves.delete(slot) {
        Ok(()) => {
            let message = ctx.message("Deleted {slot}", &[("slot", &slot_label(ctx, slot))]);
            ctx.notify(message);
        }
        Err(e) => {
            eprintln!("⚠️ Delete failed ({}): {}", slot, e);
            let message = ctx.message(
                "Delete failed: {reason}",
                &[("reason", &e.player_message())],
            );
            ctx.notify_error(message);
        }
    }
}

pub(super) fn slot_label(ctx: &GameContext, slot: &str) -> String {
    match slot {
        QUICK_SLOT => ctx.label("Quick save").to_string(),
        AUTO_SLOT => ctx.label("Autosave").to_string(),
        _ => ctx.message("Slot {number}", &[("number", slot)]),
    }
}

pub(super) fn slot_label_for(ctx: &crate::context::DrawContext, slot: &str) -> String {
    match slot {
        QUICK_SLOT => ctx.label("Quick save").to_string(),
        AUTO_SLOT => ctx.label("Autosave").to_string(),
        _ => ctx.message("Slot {number}", &[("number", slot)]),
    }
}

pub(super) fn when(ctx: &crate::context::DrawContext, saved_at: u64, now: u64) -> String {
    let elapsed = time_ago(saved_at, now);
    ctx.message(elapsed.template(), &[("n", &elapsed.count().to_string())])
}

pub(super) fn summary_of(ctx: &crate::context::DrawContext, file: &SaveFile) -> String {
    const MAX: usize = 80;

    let line = match &file.point {
        SavePoint::Line { speaker, said } => {
            let text = said.text(ctx.story.catalog());
            match speaker {
                Some(speaker) => format!(
                    "{}: {}",
                    ctx.characters.display_name(speaker, ctx.story),
                    text
                ),
                None => text,
            }
        }
        SavePoint::Choosing => ctx.label("Making a choice").to_string(),
        SavePoint::End => ctx.label("The end").to_string(),
        SavePoint::Unknown => file.summary.clone(),
    };

    if line.chars().count() <= MAX {
        return line;
    }
    let cut: String = line.chars().take(MAX - 1).collect();
    format!("{}…", cut.trim_end())
}
