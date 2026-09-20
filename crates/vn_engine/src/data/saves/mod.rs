mod dirs;
mod error;
mod file;
mod migrate;
mod store;

pub use dirs::*;
pub use error::*;
pub use file::*;
pub use migrate::*;
pub use store::*;

use std::time::{SystemTime, UNIX_EPOCH};

pub const SAVE_FORMAT_VERSION: u32 = 1;
pub const QUICK_SLOT: &str = "quick";
pub const AUTO_SLOT: &str = "auto";
pub const THUMBNAIL_WIDTH: i32 = 320;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn time_ago(saved_at: u64, now: u64) -> String {
    let seconds = now.saturating_sub(saved_at);
    let plural =
        |n: u64, unit: &str| format!("{} {}{} ago", n, unit, if n == 1 { "" } else { "s" });

    match seconds {
        0..=59 => "just now".to_string(),
        60..=3599 => plural(seconds / 60, "minute"),
        3600..=86_399 => plural(seconds / 3600, "hour"),
        86_400..=2_591_999 => plural(seconds / 86_400, "day"),
        _ => plural(seconds / 2_592_000, "month"),
    }
}
