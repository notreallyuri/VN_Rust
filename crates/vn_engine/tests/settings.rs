use std::fs;
use std::path::PathBuf;

use vn_engine::script::StoryVm;
use vn_engine::{
    ScreenState, Settings, SettingsConfig, SettingsStore, Typewriter, close_needs_confirmation,
};

fn temp_file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vn_engine_settings_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let _ = fs::remove_file(&path);
    path
}

#[test]
fn missing_or_broken_settings_use_the_defaults() {
    let path = temp_file("missing.json");
    assert_eq!(Settings::load(&path), Settings::default());

    fs::write(&path, "not json").unwrap();
    assert_eq!(Settings::load(&path), Settings::default());

    fs::write(&path, r#"{ "fullscreen": true }"#).unwrap();
    assert_eq!(
        Settings::load(&path),
        Settings {
            fullscreen: true,
            ..Settings::default()
        }
    );
    fs::remove_file(&path).unwrap();
}

#[test]
fn changes_are_saved_right_away() {
    let path = temp_file("store.json");

    let mut store = SettingsStore::load(&path);
    store.update(|_| {});
    assert!(!path.exists(), "unchanged settings aren't written");

    store.update(|s| s.text_speed = 80);
    assert_eq!(SettingsStore::load(&path).values.text_speed, 80);

    let mut memory = SettingsStore::in_memory();
    memory.update(|s| s.fullscreen = true);
    assert!(memory.values.fullscreen);
    fs::remove_file(&path).unwrap();
}

#[test]
fn typewriter_reveals_characters_over_time() {
    let text = "Héllo, world";
    let mut typewriter = Typewriter::start(text, 10, 100.0);

    assert_eq!(typewriter.visible(100.0), 0);
    assert_eq!(typewriter.visible(100.35), 3);
    assert!(!typewriter.is_done(100.5));
    assert_eq!(typewriter.visible(200.0), 12);
    assert!(typewriter.is_done(200.0));

    let mut skipped = Typewriter::start(text, 10, 100.0);
    skipped.finish();
    assert_eq!(skipped.visible(100.0), 12);

    typewriter = Typewriter::start(text, 0, 100.0);
    assert!(typewriter.is_done(100.0));
    assert_eq!(Typewriter::finished(text).visible(0.0), 12);
}

#[test]
fn text_speeds_cycle() {
    let config = SettingsConfig::default();
    assert_eq!(config.next_text_speed(20), 40);
    assert_eq!(config.next_text_speed(80), 0);
    assert_eq!(config.next_text_speed(0), 20);
    assert_eq!(config.next_text_speed(33), 20);
    assert_eq!(config.text_speed_name(0), "Instant");
    assert_eq!(config.text_speed_name(33), "33 chars/s");

    let custom = SettingsConfig::default().text_speeds([("Calm", 15), ("Brisk", 60)]);
    assert_eq!(custom.next_text_speed(60), 15);
}

#[test]
fn closing_asks_only_during_a_game() {
    let mut story = StoryVm::from_source("scene start:\n  \"x\"\n");
    let asks = |state: ScreenState, story: &StoryVm| close_needs_confirmation(&state, story);

    assert!(!asks(ScreenState::StartScreen, &story));
    assert!(!asks(ScreenState::MainMenu, &story));
    assert!(asks(ScreenState::Playing, &story));
    assert!(asks(ScreenState::TextInput, &story));
    assert!(!asks(ScreenState::Settings, &story), "no game started yet");
    assert!(!asks(ScreenState::Load, &story));

    story.advance_until_blocking();
    assert!(asks(ScreenState::Settings, &story));
    assert!(asks(ScreenState::Custom("inventory".into()), &story));
    assert!(!asks(ScreenState::MainMenu, &story));

    story.advance_until_blocking();
    assert!(!asks(ScreenState::Playing, &story), "the story is over");
    assert!(!asks(ScreenState::Settings, &story));
}
