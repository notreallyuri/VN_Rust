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

pub const SAVE_FORMAT_VERSION: u32 = 3;
pub const QUICK_SLOT: &str = "quick";
pub const AUTO_SLOT: &str = "auto";
pub const THUMBNAIL_WIDTH: i32 = 320;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elapsed {
    JustNow,
    Minutes(u64),
    Hours(u64),
    Days(u64),
    Months(u64),
}

pub fn time_ago(saved_at: u64, now: u64) -> Elapsed {
    match now.saturating_sub(saved_at) {
        0..=59 => Elapsed::JustNow,
        seconds @ 60..=3599 => Elapsed::Minutes(seconds / 60),
        seconds @ 3600..=86_399 => Elapsed::Hours(seconds / 3600),
        seconds @ 86_400..=2_591_999 => Elapsed::Days(seconds / 86_400),
        seconds => Elapsed::Months(seconds / 2_592_000),
    }
}

impl Elapsed {
    pub const MESSAGES: &'static [&'static str] = &[
        "just now",
        "1 minute ago",
        "{n} minutes ago",
        "1 hour ago",
        "{n} hours ago",
        "1 day ago",
        "{n} days ago",
        "1 month ago",
        "{n} months ago",
    ];

    pub fn template(self) -> &'static str {
        match self {
            Elapsed::JustNow => "just now",
            Elapsed::Minutes(1) => "1 minute ago",
            Elapsed::Minutes(_) => "{n} minutes ago",
            Elapsed::Hours(1) => "1 hour ago",
            Elapsed::Hours(_) => "{n} hours ago",
            Elapsed::Days(1) => "1 day ago",
            Elapsed::Days(_) => "{n} days ago",
            Elapsed::Months(1) => "1 month ago",
            Elapsed::Months(_) => "{n} months ago",
        }
    }

    pub fn count(self) -> u64 {
        match self {
            Elapsed::JustNow => 0,
            Elapsed::Minutes(n) | Elapsed::Hours(n) | Elapsed::Days(n) | Elapsed::Months(n) => n,
        }
    }
}
