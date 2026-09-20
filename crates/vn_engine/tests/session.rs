use std::fs;

use vn_engine::action::Action;
use vn_engine::data::rollback::{Rollback, RollbackConfig};
use vn_engine::data::saves::Saves;
use vn_engine::data::session::{LogEntry, SeenLines, SessionLog};
use vn_engine::data::settings::Settings;
use vn_engine::data::state::GameState;
use vn_engine::screens::playing::{HudButton, PlayingConfig};
use vn_engine::script::StoryVm;

fn line(text: &str) -> LogEntry {
    LogEntry::Line {
        speaker: None,
        text: text.into(),
    }
}

#[test]
fn the_log_keeps_order_forgets_the_oldest_and_can_be_rewound() {
    let mut log = SessionLog::new(3);
    assert!(log.is_empty());
    log.push(line("one"));
    log.push(LogEntry::Choice { text: "Go".into() });
    log.push(line("two"));
    log.push(line("three"));
    assert_eq!(log.len(), 3);
    assert_eq!(log.entries()[0], LogEntry::Choice { text: "Go".into() });

    log.show(1);
    assert_eq!(log.entries().len(), 1, "rolled back");
    log.show(3);
    assert_eq!(log.entries().len(), 3, "and forward again");
    log.show(1);
    log.push(line("new branch"));
    assert_eq!(
        log.len(),
        2,
        "a new line drops what came after the rollback"
    );
    assert_eq!(log.entries()[1], line("new branch"));

    log.replace(vec![line("a"), line("b"), line("c"), line("d")]);
    assert_eq!(log.entries(), [line("b"), line("c"), line("d")]);
    log.clear();
    assert!(log.is_empty());
}

#[test]
fn log_entries_serialize_compactly() {
    let json = serde_json::to_string(&LogEntry::Choice { text: "Go".into() }).unwrap();
    assert_eq!(json, r#"{"choice":{"text":"Go"}}"#);
}

#[test]
fn seen_lines_persist_between_sessions() {
    let dir = std::env::temp_dir().join(format!("vn_engine_seen_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let path = dir.join("seen.json");

    let mut seen = SeenLines::load(&path);
    assert!(seen.is_empty());
    seen.insert(7);
    seen.insert(7);
    seen.insert(42);
    assert_eq!(seen.len(), 2);
    seen.save();

    let again = SeenLines::load(&path);
    assert!(again.contains(7) && again.contains(42) && !again.contains(1));

    fs::write(&path, "not json").unwrap();
    assert!(
        SeenLines::load(&path).is_empty(),
        "a broken file starts over"
    );
    let _ = fs::remove_dir_all(&dir);

    let mut memory = SeenLines::in_memory();
    memory.insert(1);
    memory.save();
    assert!(memory.path().is_none());
}

#[test]
fn checkpoints_remember_the_log_length() {
    let mut story = StoryVm::from_source("scene a:\n  \"one\"\n  \"two\"\n");
    let mut state = GameState::default();
    let mut rollback = Rollback::new(RollbackConfig::default());

    story.advance_until_blocking();
    rollback.record_with_log(&story, &state, Some(1));
    story.advance_until_blocking();
    rollback.record_with_log(&story, &state, Some(2));
    assert_eq!(rollback.log_len(), Some(2));

    assert!(rollback.back(&mut story, &mut state));
    assert_eq!(rollback.log_len(), Some(1));

    let json = serde_json::to_value(&rollback.history()[0]).unwrap();
    let mut old = json.clone();
    old.as_object_mut().unwrap().remove("log_len");
    let old: vn_engine::data::rollback::Checkpoint = serde_json::from_value(old).unwrap();
    assert_eq!(old.log_len, None, "older saves have no log length");
}

#[test]
fn saves_carry_the_log() {
    let dir = std::env::temp_dir().join(format!("vn_engine_log_save_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let saves = Saves::new(&dir, "Test");
    let mut story = StoryVm::from_source("scene a:\n  \"one\"\n");
    story.advance_until_blocking();

    let mut file = saves.capture(&story, &GameState::default()).unwrap();
    assert!(file.log.is_empty());
    file.log = vec![line("one")];
    saves.write("1", &file).unwrap();
    assert_eq!(saves.read("1").unwrap().log, [line("one")]);

    let json = fs::read_to_string(dir.join("1.json")).unwrap();
    assert!(json.contains("\"log\""));
    file.log.clear();
    saves.write("2", &file).unwrap();
    let json = fs::read_to_string(dir.join("2.json")).unwrap();
    assert!(!json.contains("\"log\""), "an empty log isn't written");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_default_hud_is_log_auto_and_skip() {
    let config = PlayingConfig::default();
    let labels: Vec<&str> = config.hud.iter().map(|b| b.label.as_str()).collect();
    assert_eq!(labels, ["Log", "Auto", "Skip"]);
    assert!(matches!(config.hud[1].action, Action::ToggleAuto));

    let custom = PlayingConfig::default().hud_item(HudButton::new("Menu", Action::Resume));
    assert_eq!(custom.hud.len(), 1, "a game's own HUD replaces the default");

    let settings = Settings::default();
    assert_eq!(
        config.auto_delay(&settings, "0123456789"),
        1.5 + 10.0 * 0.02
    );
}
