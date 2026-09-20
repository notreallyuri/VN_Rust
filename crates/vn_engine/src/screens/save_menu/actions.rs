use crate::GameContext;
use crate::data::saves::{AUTO_SLOT, QUICK_SLOT};

pub(super) fn save_to(ctx: &mut GameContext, slot: &str) {
    match ctx.save(slot) {
        Ok(()) => ctx.notify(format!("Saved to {}", slot_label(slot))),
        Err(e) => {
            eprintln!("⚠️ Save failed ({}): {}", slot, e);
            ctx.notify_error(format!("Save failed: {}", e.player_message()));
        }
    }
}

pub(super) fn load_from(ctx: &mut GameContext, slot: &str) -> bool {
    match ctx.load(slot) {
        Ok(_) => {
            ctx.notify(format!("Loaded {}", slot_label(slot)));
            true
        }
        Err(e) => {
            eprintln!("⚠️ Load failed ({}): {}", slot, e);
            ctx.notify_error(format!("Load failed: {}", e.player_message()));
            false
        }
    }
}

pub(super) fn delete_slot(ctx: &mut GameContext, slot: &str) {
    match ctx.saves.delete(slot) {
        Ok(()) => ctx.notify(format!("Deleted {}", slot_label(slot))),
        Err(e) => {
            eprintln!("⚠️ Delete failed ({}): {}", slot, e);
            ctx.notify_error(format!("Delete failed: {}", e.player_message()));
        }
    }
}

pub(super) fn slot_label(slot: &str) -> String {
    match slot {
        QUICK_SLOT => "Quick save".to_string(),
        AUTO_SLOT => "Autosave".to_string(),
        _ => format!("Slot {}", slot),
    }
}
