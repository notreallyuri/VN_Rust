use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};
use vn_engine::saves::time_ago;
use vn_engine::script::{Event, StoryVm, Value, VmError};
use vn_engine::{
    AUTO_SLOT, GameState, LoadWarning, SAVE_FORMAT_VERSION, SaveError, Saves, THUMBNAIL_WIDTH,
    default_saves_dir, slug,
};

const STORY: &str = r#"
scene intro:
  set met_mary = true
  mary "hello"
  jump middle

scene middle:
  show mary neutral
  "middle line"
  "last line"
"#;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Inventory {
    items: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Affection {
    mary: i32,
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "vn_engine_saves_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fresh_state() -> GameState {
    let mut state = GameState::default();
    state.insert(Inventory::default());
    state.insert(Affection::default());
    state
}

fn story_at(text: &str) -> StoryVm {
    let mut vm = StoryVm::from_source(STORY);
    loop {
        match vm.advance_until_blocking() {
            Event::Say { text: t, .. } if t == text => return vm,
            Event::End => panic!("never reached {:?}", text),
            _ => {}
        }
    }
}

fn played_game() -> (StoryVm, GameState) {
    let vm = story_at("middle line");
    let mut state = fresh_state();
    state
        .get_mut::<Inventory>()
        .items
        .insert("letter".into(), 2);
    state.get_mut::<Affection>().mary = 3;
    (vm, state)
}

#[test]
fn round_trip() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state_before) = played_game();

    saves.save("1", &vm, &state_before).unwrap();

    let mut loaded_vm = StoryVm::from_source(STORY);
    let mut loaded_state = fresh_state();
    let report = saves.load("1", &mut loaded_vm, &mut loaded_state).unwrap();

    assert!(report.warnings.is_empty());
    assert_eq!(loaded_vm.current(), vm.current());
    assert_eq!(loaded_vm.current_scene(), Some("middle"));
    assert_eq!(loaded_vm.variable("met_mary"), Some(&Value::Bool(true)));
    assert_eq!(loaded_vm.active_characters(), vm.active_characters());
    assert_eq!(
        loaded_state.get::<Inventory>(),
        state_before.get::<Inventory>()
    );
    assert_eq!(loaded_state.get::<Affection>().mary, 3);
    assert_eq!(
        loaded_vm.advance(),
        Event::Say {
            speaker: None,
            text: "last line".into()
        }
    );
}

#[test]
fn save_file_is_readable_json_with_a_summary() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();

    let file = saves.read("1").unwrap();
    assert_eq!(file.format_version, SAVE_FORMAT_VERSION);
    assert_eq!(file.game, "Test Game");
    assert_eq!(file.summary, "middle line");
    assert!(file.state.contains_key("Inventory"));
    assert!(file.state.contains_key("Affection"));

    let raw = fs::read_to_string(dir.0.join("1.json")).unwrap();
    assert!(raw.contains("\"scene\": \"middle\""));
}

#[test]
fn save_leaves_no_temp_file() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();
    saves.save("1", &vm, &state).unwrap();

    let names: Vec<String> = fs::read_dir(&dir.0)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(names, ["1.json"]);
}

#[test]
fn empty_slot() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");

    assert!(matches!(saves.read("3"), Err(SaveError::Empty { slot }) if slot == "3"));
    assert!(saves.slot("3").is_empty());
}

#[test]
fn corrupt_file() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    fs::create_dir_all(&dir.0).unwrap();
    fs::write(dir.0.join("1.json"), "{ not json").unwrap();

    assert!(matches!(saves.read("1"), Err(SaveError::Corrupt { .. })));
}

#[test]
fn truncated_file_is_corrupt() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();

    let path = dir.0.join("1.json");
    let full = fs::read_to_string(&path).unwrap();
    fs::write(&path, &full[..full.len() / 2]).unwrap();

    assert!(matches!(saves.read("1"), Err(SaveError::Corrupt { .. })));
}

#[test]
fn newer_format_is_rejected() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    let mut file = saves.capture(&vm, &state).unwrap();
    file.format_version = SAVE_FORMAT_VERSION + 1;
    saves.write("1", &file).unwrap();

    assert!(matches!(
        saves.read("1"),
        Err(SaveError::NewerFormat { found, supported })
            if found == SAVE_FORMAT_VERSION + 1 && supported == SAVE_FORMAT_VERSION
    ));
}

#[test]
fn other_game_is_rejected() {
    let dir = TempDir::new();
    let (vm, state) = played_game();
    Saves::new(&dir.0, "Another Game")
        .save("1", &vm, &state)
        .unwrap();

    assert!(matches!(
        Saves::new(&dir.0, "Test Game").read("1"),
        Err(SaveError::OtherGame { found, .. }) if found == "Another Game"
    ));
}

#[test]
fn slot_names_cannot_escape_the_directory() {
    let saves = Saves::new("saves", "Test Game");

    for slot in ["", "../evil", "a/b", "c:\\x", "slot 1"] {
        assert!(
            matches!(saves.path(slot), Err(SaveError::InvalidSlot(_))),
            "{:?}",
            slot
        );
    }
    assert!(saves.path("quick").is_ok());
    assert!(saves.path("slot_2-b").is_ok());
}

#[test]
fn failed_load_leaves_the_game_untouched() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    let mut file = saves.capture(&vm, &state).unwrap();
    file.state
        .insert("Affection".into(), serde_json::json!({ "mary": "lots" }));
    saves.write("1", &file).unwrap();

    let mut current_vm = story_at("hello");
    let mut current_state = fresh_state();
    current_state.get_mut::<Affection>().mary = 7;

    let result = saves.load("1", &mut current_vm, &mut current_state);

    assert!(matches!(result, Err(SaveError::State(ref e)) if e.key == "Affection"));
    assert_eq!(current_state.get::<Affection>().mary, 7);
    assert_eq!(current_vm.current_scene(), Some("intro"));
    assert_eq!(
        current_vm.current(),
        Some(&Event::Say {
            speaker: Some("mary".into()),
            text: "hello".into()
        })
    );
}

#[test]
fn missing_scene_fails_without_touching_state() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();

    let without_middle = STORY.split("scene middle:").next().unwrap();
    let mut other_vm = StoryVm::from_source(without_middle);
    let mut other_state = fresh_state();

    let result = saves.load("1", &mut other_vm, &mut other_state);

    assert!(matches!(
        result,
        Err(SaveError::Story(VmError::UnknownScene(ref s))) if s == "middle"
    ));
    assert_eq!(other_state.get::<Affection>().mary, 0);
}

#[test]
fn edited_scene_loads_with_a_warning() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();

    let edited = STORY.replace("\"middle line\"", "\"middle line, revised\"");
    let mut loaded_vm = StoryVm::from_source(&edited);
    let mut loaded_state = fresh_state();
    let report = saves.load("1", &mut loaded_vm, &mut loaded_state).unwrap();

    assert_eq!(
        report.warnings,
        [LoadWarning::SceneRestarted {
            scene: "middle".into()
        }]
    );
    assert_eq!(loaded_state.get::<Affection>().mary, 3);
}

#[test]
fn state_added_or_removed_since_the_save() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, _) = played_game();

    let mut old_state = GameState::default();
    old_state.insert(Affection { mary: 5 });
    old_state.insert(BTreeMap::<String, i32>::new());
    saves.save("1", &vm, &old_state).unwrap();

    let mut loaded_vm = StoryVm::from_source(STORY);
    let mut new_state = fresh_state();
    new_state.get_mut::<Inventory>().items.insert("x".into(), 1);
    let report = saves.load("1", &mut loaded_vm, &mut new_state).unwrap();

    assert_eq!(
        report.warnings,
        [
            LoadWarning::MissingState {
                key: "Inventory".into()
            },
            LoadWarning::UnknownState {
                key: "BTreeMap".into()
            },
        ]
    );
    assert_eq!(new_state.get::<Affection>().mary, 5);
    assert!(new_state.get::<Inventory>().items.is_empty());
}

#[test]
fn latest_picks_the_most_recent_save() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();

    for (slot, saved_at) in [("1", 100), ("quick", 300), ("2", 200)] {
        let mut file = saves.capture(&vm, &state).unwrap();
        file.saved_at = saved_at;
        saves.write(slot, &file).unwrap();
    }
    fs::write(dir.0.join("broken.json"), "nope").unwrap();

    assert_eq!(saves.latest().map(|(slot, _)| slot), Some("quick".into()));
}

#[test]
fn delete_removes_a_slot() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();

    saves.delete("1").unwrap();
    saves.delete("1").unwrap();
    assert!(saves.slot("1").is_empty());
}

#[test]
fn reset_restores_initial_values() {
    let mut state = fresh_state();
    state.get_mut::<Affection>().mary = 9;
    state.reset();
    assert_eq!(state.get::<Affection>().mary, 0);
}

#[test]
#[should_panic(expected = "is used by both")]
fn state_key_collisions_are_rejected() {
    mod a {
        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        pub struct Stats;
    }
    mod b {
        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        pub struct Stats;
    }

    let mut state = GameState::default();
    state.insert(a::Stats);
    state.insert(b::Stats);
}

#[test]
fn relative_times() {
    assert_eq!(time_ago(1000, 1030), "just now");
    assert_eq!(time_ago(1000, 1060), "1 minute ago");
    assert_eq!(time_ago(1000, 1000 + 5 * 60), "5 minutes ago");
    assert_eq!(time_ago(0, 2 * 3600), "2 hours ago");
    assert_eq!(time_ago(0, 86_400), "1 day ago");
    assert_eq!(time_ago(0, 90 * 86_400), "3 months ago");
    assert_eq!(time_ago(2000, 1000), "just now");
}

#[test]
fn rollback_history_is_stored_in_the_file() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();

    let mut rollback = vn_engine::Rollback::default();
    let mut replay = StoryVm::from_source(STORY);
    replay.advance_until_blocking();
    rollback.record(&replay, &state);
    rollback.record(&vm, &state);

    let mut file = saves.capture(&vm, &state).unwrap();
    file.rollback = rollback.history();
    saves.write("1", &file).unwrap();

    let read = saves.read("1").unwrap();
    assert_eq!(read.rollback.len(), 2);
    assert_eq!(read.rollback, file.rollback);

    let mut restored = vn_engine::Rollback::default();
    let mut loaded = StoryVm::from_source(STORY);
    vn_engine::saves::apply(&read, &mut loaded, &mut fresh_state()).unwrap();
    assert_eq!(restored.restore_history(read.rollback, &loaded), 2);
    assert!(restored.can_go_back());
}

#[test]
fn saves_without_history_still_load() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();

    saves.save("1", &vm, &state).unwrap();
    let json = fs::read_to_string(dir.0.join("1.json")).unwrap();
    assert!(
        !json.contains("\"rollback\""),
        "empty history isn't written"
    );
    assert!(saves.read("1").unwrap().rollback.is_empty());
}

#[test]
fn thumbnails_are_written_scaled_and_removed_with_their_slot() {
    use vn_engine::raylib::prelude::{Color, Image};

    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();

    let screen = Image::gen_image_color(1280, 720, Color::RED);
    saves.write_thumbnail("1", Some(&screen)).unwrap();
    let path = saves.thumbnail_path("1").unwrap();
    assert_eq!(path, dir.0.join("1.png"));
    let written = Image::load_image(&path.to_string_lossy()).unwrap();
    assert_eq!(
        (written.width(), written.height()),
        (THUMBNAIL_WIDTH, THUMBNAIL_WIDTH * 9 / 16)
    );

    saves.write_thumbnail("1", None).unwrap();
    assert!(
        !path.exists(),
        "a save without a screenshot drops the old one"
    );

    saves.write_thumbnail("1", Some(&screen)).unwrap();
    let before = saves.generation();
    saves.delete("1").unwrap();
    assert!(!path.exists());
    assert!(!dir.0.join("1.json").exists());
    assert!(saves.generation() > before);
    saves.delete("1").unwrap();
}

#[test]
fn autosave_is_opt_in_on_saves() {
    assert!(!Saves::new("saves", "Test Game").autosaves());
    assert!(
        Saves::new("saves", "Test Game")
            .with_autosave(true)
            .autosaves()
    );
    assert_eq!(AUTO_SLOT, "auto");
}

#[test]
fn slugs_name_the_platform_save_directory() {
    assert_eq!(slug("God Is Watching"), "god_is_watching");
    assert_eq!(slug("  Ren'Py -- Test!  "), "ren_py_test");
    assert_eq!(slug("Café 2"), "café_2");
    assert_eq!(slug("!!!"), "game");

    let dir = default_saves_dir("God Is Watching");
    assert!(dir.ends_with("god_is_watching"), "{}", dir.display());
}
