use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use novn::data::rollback::Checkpoint;
use novn::data::saves::{
    AUTO_SLOT, LoadWarning, SAVE_FORMAT_VERSION, SaveError, Saves, THUMBNAIL_WIDTH,
    default_saves_dir, slug,
};
use novn::data::saves::{Elapsed, SavePoint, time_ago};
use novn::data::session::Spoken;
use novn::data::state::GameState;
use novn::script::{Event, StoryVm, Value, VmError};
use serde::{Deserialize, Serialize};

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
fn a_save_carries_what_the_game_pushed_onto_its_visuals() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();

    let mut file = saves.capture(&vm, &state).unwrap();
    assert!(file.visuals.is_empty(), "a save captures none of its own");
    file.visuals = BTreeMap::from([(
        "mary".to_string(),
        BTreeMap::from([("blush".to_string(), 0.75)]),
    )]);
    file.rollback = vec![Checkpoint {
        story: vm.snapshot(),
        state: state.to_json().unwrap(),
        barrier: false,
        log_len: None,
        visuals: file.visuals.clone(),
    }];
    saves.write("1", &file).unwrap();

    let read = saves.read("1").unwrap();
    assert_eq!(read.visuals["mary"]["blush"], 0.75);
    assert_eq!(
        read.rollback[0].visuals, read.visuals,
        "and every step keeps its own"
    );

    let text = fs::read_to_string(dir.0.join("saves").join("1.json"))
        .or_else(|_| fs::read_to_string(saves.path("1").unwrap()))
        .unwrap();
    assert!(text.contains("\"visuals\""), "{text}");
}

#[test]
fn a_save_written_before_visuals_existed_still_loads() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();

    let path = saves.path("1").unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(
        !text.contains("visuals"),
        "an empty map is not written at all"
    );

    let read = saves.read("1").unwrap();
    assert!(read.visuals.is_empty());
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
    assert!(file.state.contains_key("Inventory"));
    assert!(file.state.contains_key("Affection"));

    let SavePoint::Line { speaker, said } = &file.point else {
        panic!("expected a line, got {:?}", file.point);
    };
    assert_eq!(speaker.as_deref(), None);
    assert_eq!(said.source, "middle line");
    assert_eq!(said.text(None), "middle line");

    let raw = fs::read_to_string(dir.0.join("1.json")).unwrap();
    assert!(raw.contains("\"scene\": \"middle\""));
    assert!(
        !raw.contains("\"summary\""),
        "the translated line should not be written any more:\n{}",
        raw
    );
}

#[test]
fn a_save_point_keeps_the_values_a_line_was_read_with() {
    let source = "Hello, {player_name}. You have {coins} coins.";
    let mut variables = std::collections::HashMap::new();
    variables.insert("player_name".to_string(), Value::String("Mary".into()));
    variables.insert("coins".to_string(), Value::Int(3));

    let said = Spoken::with_variables("01.story", source, &variables);
    assert_eq!(said.fields.len(), 2, "only what the line refers to");
    assert_eq!(said.text(None), "Hello, Mary. You have 3 coins.");

    variables.insert("coins".to_string(), Value::Int(99));
    assert_eq!(
        said.text(None),
        "Hello, Mary. You have 3 coins.",
        "a later change to the variable does not rewrite the log"
    );
}

#[test]
fn a_save_from_the_old_format_still_shows_its_summary() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();

    let path = dir.0.join("1.json");
    let mut json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let file = json.as_object_mut().unwrap();
    file.insert("format_version".into(), 1.into());
    file.insert("summary".into(), "middle line".into());
    file.remove("point");
    fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

    let file = saves.read("1").unwrap();
    assert_eq!(file.point, SavePoint::Unknown);
    assert_eq!(file.summary, "middle line");
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
    let shown = |saved_at, now| {
        let elapsed = time_ago(saved_at, now);
        novn::ui::labels::fill(elapsed.template(), &[("n", &elapsed.count().to_string())])
    };

    assert_eq!(time_ago(1000, 1030), Elapsed::JustNow);
    assert_eq!(time_ago(1000, 1060), Elapsed::Minutes(1));
    assert_eq!(time_ago(0, 2 * 3600), Elapsed::Hours(2));
    assert_eq!(time_ago(0, 86_400), Elapsed::Days(1));
    assert_eq!(time_ago(0, 90 * 86_400), Elapsed::Months(3));
    assert_eq!(time_ago(2000, 1000), Elapsed::JustNow);

    assert_eq!(shown(1000, 1030), "just now");
    assert_eq!(shown(1000, 1060), "1 minute ago");
    assert_eq!(shown(1000, 1000 + 5 * 60), "5 minutes ago");
    assert_eq!(shown(0, 2 * 3600), "2 hours ago");
    assert_eq!(shown(0, 90 * 86_400), "3 months ago");
}

#[test]
fn every_relative_time_has_a_translatable_message() {
    let templates = [
        Elapsed::JustNow,
        Elapsed::Minutes(1),
        Elapsed::Minutes(5),
        Elapsed::Hours(1),
        Elapsed::Hours(5),
        Elapsed::Days(1),
        Elapsed::Days(5),
        Elapsed::Months(1),
        Elapsed::Months(5),
    ];
    for elapsed in templates {
        assert!(
            Elapsed::MESSAGES.contains(&elapsed.template()),
            "{:?} shows {:?}, which novn translate never sees",
            elapsed,
            elapsed.template()
        );
    }
}

#[test]
fn rollback_history_is_stored_in_the_file() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();

    let mut rollback = novn::data::rollback::Rollback::default();
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

    let mut restored = novn::data::rollback::Rollback::default();
    let mut loaded = StoryVm::from_source(STORY);
    novn::data::saves::apply(&read, &mut loaded, &mut fresh_state()).unwrap();
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
    use novn::raylib::prelude::{Color, Image};

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

fn write_old_save(dir: &TempDir) {
    let (vm, state) = played_game();
    let saves = Saves::new(&dir.0, "Test Game");
    let mut file = saves.capture(&vm, &state).unwrap();
    file.rollback = vec![Checkpoint {
        visuals: Default::default(),
        story: vm.snapshot(),
        state: file.state.clone(),
        barrier: false,
        log_len: None,
    }];
    saves.write("1", &file).unwrap();

    let path = dir.0.join("1.json");
    let raw = fs::read_to_string(&path)
        .unwrap()
        .replace("\"Inventory\"", "\"Bag\"")
        .replace("\"items\"", "\"things\"")
        .replace("\"met_mary\"", "\"met\"");
    fs::write(&path, raw).unwrap();
}

fn migrating_saves(dir: &TempDir) -> Saves {
    Saves::new(&dir.0, "Test Game")
        .with_version(2)
        .with_migration(0, |save| {
            save.rename_state("Bag", "Inventory");
            save.rename_variable("met", "met_mary");
            Ok(())
        })
        .with_migration(1, |save| {
            save.state("Inventory", |inventory| {
                let things = inventory
                    .as_object_mut()
                    .and_then(|o| o.remove("things"))
                    .ok_or("no things")?;
                inventory["items"] = things;
                Ok(())
            })
        })
}

#[test]
fn old_saves_are_migrated_on_load() {
    let dir = TempDir::new();
    write_old_save(&dir);

    assert!(matches!(
        Saves::new(&dir.0, "Test Game").read("1"),
        Ok(file) if !file.state.contains_key("Inventory")
    ));

    let saves = migrating_saves(&dir);
    let file = saves.read("1").unwrap();
    assert_eq!(file.game_version, 2);
    assert_eq!(file.format_version, SAVE_FORMAT_VERSION);
    assert!(!file.state.contains_key("Bag"));
    assert_eq!(file.rollback[0].state, file.state);
    assert!(file.rollback[0].story.variables.contains_key("met_mary"));

    let mut vm = StoryVm::from_source(STORY);
    let mut state = fresh_state();
    let report = saves.load("1", &mut vm, &mut state).unwrap();
    assert!(report.warnings.is_empty());
    assert_eq!(state.get::<Inventory>().items["letter"], 2);
    assert_eq!(vm.variable("met_mary"), Some(&Value::Bool(true)));
}

#[test]
fn saves_record_the_game_version() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game").with_version(3);
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();

    let raw = fs::read_to_string(dir.0.join("1.json")).unwrap();
    assert!(raw.contains("\"game_version\": 3"));
    assert_eq!(saves.read("1").unwrap().game_version, 3);
}

#[test]
fn versions_without_a_migration_load_unchanged() {
    let dir = TempDir::new();
    let (vm, state) = played_game();
    Saves::new(&dir.0, "Test Game")
        .save("1", &vm, &state)
        .unwrap();

    let file = Saves::new(&dir.0, "Test Game")
        .with_version(4)
        .read("1")
        .unwrap();
    assert_eq!(file.game_version, 4);
    assert!(file.state.contains_key("Inventory"));
}

#[test]
fn newer_game_version_is_rejected() {
    let dir = TempDir::new();
    let (vm, state) = played_game();
    Saves::new(&dir.0, "Test Game")
        .with_version(2)
        .save("1", &vm, &state)
        .unwrap();

    let result = Saves::new(&dir.0, "Test Game").with_version(1).read("1");
    assert!(matches!(
        result,
        Err(SaveError::NewerGameVersion {
            found: 2,
            supported: 1
        })
    ));
}

#[test]
fn failed_migration_reports_its_version() {
    let dir = TempDir::new();
    let (vm, state) = played_game();
    Saves::new(&dir.0, "Test Game")
        .save("1", &vm, &state)
        .unwrap();

    let saves = Saves::new(&dir.0, "Test Game")
        .with_version(2)
        .with_migration(0, |_| Ok(()))
        .with_migration(1, |_| Err("bad data".into()));
    let error = saves.read("1").unwrap_err();
    assert!(matches!(
        &error,
        SaveError::Migration { from: 1, message, .. } if message == "bad data"
    ));
    assert_eq!(
        error.player_message(),
        "This save couldn't be updated for this version of the game."
    );
}

#[test]
fn a_save_from_the_old_format_keeps_its_log() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();

    let path = dir.0.join("1.json");
    let mut json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let file = json.as_object_mut().unwrap();
    file.insert("format_version".into(), 1.into());
    file.insert("summary".into(), "middle line".into());
    file.remove("point");
    file.insert(
        "log".into(),
        serde_json::json!([
            {"line": {"speaker": null, "text": "October, 1903."}},
            {"line": {"speaker": "mary", "text": "Your hot water, miss."}},
            {"choice": {"text": "Read everything first"}},
        ]),
    );
    fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

    let file = saves
        .read("1")
        .expect("a save written before the log kept source text must still load");
    assert_eq!(
        file.format_version, SAVE_FORMAT_VERSION,
        "and is brought up to date"
    );
    let shown: Vec<String> = file.log.iter().map(|e| e.said().text(None)).collect();
    assert_eq!(
        shown,
        [
            "October, 1903.",
            "Your hot water, miss.",
            "Read everything first"
        ],
        "old lines show exactly as they were written"
    );
}

#[test]
fn a_save_made_mid_choice_before_options_carried_their_index_still_loads() {
    let dir = TempDir::new();
    let saves = Saves::new(&dir.0, "Test Game");
    let (vm, state) = played_game();
    saves.save("1", &vm, &state).unwrap();

    let path = dir.0.join("1.json");
    let mut json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let file = json.as_object_mut().unwrap();
    file.insert("format_version".into(), 2.into());
    file["story"]["current"] = serde_json::json!({
        "Choice": { "options": ["Open the letter", "Leave it sealed"] }
    });
    fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).unwrap();

    let file = saves
        .read("1")
        .expect("a save written before options carried their index must still load");
    assert_eq!(file.format_version, SAVE_FORMAT_VERSION);

    let Some(Event::Choice { options }) = &file.story.current else {
        panic!("the save is waiting on a choice");
    };
    assert_eq!(options[1].text, "Leave it sealed");
    assert_eq!(options[1].index, 1, "the position becomes the index");
    assert!(options[1].enabled);
}
