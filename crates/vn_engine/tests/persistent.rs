use std::cell::RefCell;
use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use vn_engine::action::Action;
use vn_engine::context::{DrawContext, GameContext};
use vn_engine::data::persistent::{PERSISTENT_FILE_NAME, Persistent};
use vn_engine::data::saves::Saves;
use vn_engine::data::session::{background_key, character_key, music_key};
use vn_engine::data::state::GameState;
use vn_engine::screen::{Screen, ScreenState};
use vn_engine::screen_manager::{ScreenFactory, ScreenStateManager};
use vn_engine::screens::playing::{PlayingConfig, PlayingScreen};
use vn_engine::script::StoryVm;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Achievements {
    unlocked: BTreeSet<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Endings {
    reached: u32,
}

fn file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "vn_engine_persistent_{}_{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    dir.join(PERSISTENT_FILE_NAME)
}

fn open(path: &PathBuf) -> Persistent {
    let mut store = Persistent::in_memory();
    store.insert(Achievements::default());
    store.insert(Endings::default());
    store.load(path);
    store
}

#[test]
fn values_outlive_the_session_that_wrote_them() {
    let path = file("survive");
    let mut store = open(&path);
    store
        .get_mut::<Achievements>()
        .unlocked
        .insert("ending_report".into());
    store.get_mut::<Endings>().reached = 2;
    store.save();

    let store = open(&path);
    assert!(
        store
            .get::<Achievements>()
            .unlocked
            .contains("ending_report")
    );
    assert_eq!(store.get::<Endings>().reached, 2);
    assert!(!path.with_extension("json.tmp").exists());
}

#[test]
fn nothing_is_written_until_a_value_is_borrowed_mutably() {
    let path = file("lazy");
    let mut store = open(&path);
    let _ = store.get::<Endings>();
    store.save();
    assert!(!path.exists());
}

#[test]
fn values_from_types_the_game_no_longer_registers_are_kept() {
    let path = file("unknown");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"version":1,"values":{"Endings":{"reached":1},"Retired":{"kept":true}}}"#,
    )
    .unwrap();

    let mut store = open(&path);
    store.get_mut::<Endings>().reached = 3;
    store.save();

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(json["values"]["Retired"]["kept"], true);
    assert_eq!(json["values"]["Endings"]["reached"], 3);
}

#[test]
fn a_value_that_no_longer_fits_its_type_falls_back_alone_and_is_backed_up() {
    let path = file("reshaped");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let old = r#"{"version":1,"values":{"Endings":{"reached":"many"},"Achievements":{"unlocked":["thorough_reader"]}}}"#;
    fs::write(&path, old).unwrap();

    let store = open(&path);
    assert_eq!(store.get::<Endings>(), &Endings::default());
    assert!(
        store
            .get::<Achievements>()
            .unlocked
            .contains("thorough_reader")
    );
    assert_eq!(
        fs::read_to_string(path.with_extension("json.bak")).unwrap(),
        old
    );
}

#[test]
fn a_file_from_a_newer_version_is_read_but_never_overwritten() {
    let path = file("newer");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let newer = r#"{"version":99,"values":{"Endings":{"reached":7}}}"#;
    fs::write(&path, newer).unwrap();

    let mut store = open(&path);
    assert_eq!(store.get::<Endings>().reached, 7);
    store.get_mut::<Endings>().reached = 8;
    store.save();
    assert_eq!(fs::read_to_string(&path).unwrap(), newer);
}

#[test]
fn a_damaged_file_is_backed_up_and_replaced_on_the_next_write() {
    let path = file("damaged");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "{ not json").unwrap();

    let mut store = open(&path);
    assert_eq!(store.get::<Endings>(), &Endings::default());
    assert_eq!(
        fs::read_to_string(path.with_extension("json.bak")).unwrap(),
        "{ not json"
    );
    store.get_mut::<Endings>().reached = 1;
    store.save();
    assert_eq!(open(&path).get::<Endings>().reached, 1);
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Chapter {
    number: u32,
}

type Step = Box<dyn FnOnce(&mut GameContext) -> Option<ScreenState>>;

struct Script(Rc<RefCell<VecDeque<Step>>>);

impl Screen for Script {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        let step = self.0.borrow_mut().pop_front();
        step.map_or(Some(ScreenState::Quit), |step| step(&mut ctx))
    }

    fn draw(&self, _: &mut RaylibDrawHandle, _: &DrawContext) {}
}

struct Scripted(Rc<RefCell<VecDeque<Step>>>);

impl ScreenFactory for Scripted {
    fn create_screen(&self, _: &ScreenState) -> Option<Box<dyn Screen>> {
        Some(Box::new(Script(Rc::clone(&self.0))))
    }
}

fn unlocked(ctx: &GameContext) -> Vec<String> {
    ctx.persistent
        .get::<Achievements>()
        .unlocked
        .iter()
        .cloned()
        .collect()
}

fn unlock(ctx: &mut GameContext, key: &str) {
    ctx.persistent
        .get_mut::<Achievements>()
        .unlocked
        .insert(key.into());
}

fn chapter(ctx: &GameContext) -> u32 {
    ctx.state.get::<Chapter>().number
}

fn set_chapter(ctx: &mut GameContext, number: u32) {
    ctx.state.get_mut::<Chapter>().number = number;
}

#[test]
#[ignore = "opens a window; run with --ignored --test-threads=1 on a machine with a display"]
fn new_game_loading_and_rollback_leave_persistent_values_alone() {
    let path = file("engine");
    let dir = path.parent().unwrap().to_path_buf();
    let steps: Vec<Step> = vec![
        Box::new(|ctx| {
            ctx.story.advance();
            set_chapter(ctx, 1);
            ctx.rollback.record_with_log(ctx.story, ctx.state, None);
            unlock(ctx, "a");
            ctx.save("1").unwrap();
            None
        }),
        Box::new(|ctx| Action::NewGame.run(ctx)),
        Box::new(|ctx| {
            assert_eq!(chapter(ctx), 0, "New Game resets game state");
            assert_eq!(unlocked(ctx), ["a"], "New Game keeps persistent values");
            unlock(ctx, "b");
            ctx.load("1").unwrap();
            None
        }),
        Box::new(|ctx| {
            assert_eq!(chapter(ctx), 1, "loading restores game state");
            assert_eq!(unlocked(ctx), ["a", "b"], "loading keeps persistent values");
            set_chapter(ctx, 2);
            ctx.story.advance();
            ctx.rollback.record_with_log(ctx.story, ctx.state, None);
            unlock(ctx, "c");
            assert!(ctx.rollback.back(ctx.story, ctx.state));
            assert_eq!(chapter(ctx), 1, "rollback restores game state");
            assert_eq!(
                unlocked(ctx),
                ["a", "b", "c"],
                "rollback keeps persistent values"
            );
            Some(ScreenState::Quit)
        }),
    ];
    let steps = Rc::new(RefCell::new(VecDeque::from(steps)));

    let (mut rl, thread) = raylib::init().size(64, 64).title("persistent").build();
    rl.set_trace_log(TraceLogLevel::LOG_WARNING);
    let story = StoryVm::from_source("scene a:\n  \"one\"\n  \"two\"\n");
    let mut manager = ScreenStateManager::with_story(
        &mut rl,
        &thread,
        ScreenState::Playing,
        Box::new(Scripted(Rc::clone(&steps))),
        dir.clone(),
        story,
    )
    .unwrap();
    let mut state = GameState::default();
    state.insert(Chapter::default());
    manager.world.state = state;
    manager.world.saves = Saves::new(&dir, "Test");
    manager.world.persistent.insert(Achievements::default());
    manager.world.persistent.load(&path);

    while !manager.quit_requested() {
        manager.update(&mut rl, &thread);
    }
    manager.world.persistent.save();
    assert!(steps.borrow().is_empty(), "every step ran");

    let stored = open(&path);
    assert_eq!(
        stored.get::<Achievements>().unlocked,
        BTreeSet::from(["a".into(), "b".into(), "c".into()])
    );
    let saved = manager.world.saves.read("1").unwrap();
    assert!(saved.state.contains_key("Chapter"));
    assert!(
        !saved.state.contains_key("Achievements"),
        "a save never holds persistent values"
    );
}

#[test]
fn a_game_can_mark_its_own_art_seen_and_it_persists() {
    let path = file("marked");
    let mut store = open(&path);
    store.mark_seen("cg:kitchen_01");
    store.mark_seen("cg:kitchen_01");
    store.mark_seen("ending_card:keeper");
    assert!(store.has_seen("cg:kitchen_01"));
    assert!(!store.has_seen("cg:never"));
    assert_eq!(store.seen_art().len(), 2);
    store.save();

    let store = open(&path);
    assert_eq!(
        store.seen_art().keys().collect::<Vec<_>>(),
        ["cg:kitchen_01", "ending_card:keeper"]
    );
}

struct PlayingFactory;

impl ScreenFactory for PlayingFactory {
    fn create_screen(&self, _: &ScreenState) -> Option<Box<dyn Screen>> {
        Some(Box::new(PlayingScreen::new(Rc::new(
            PlayingConfig::default(),
        ))))
    }
}

#[test]
#[ignore = "opens a window; run with --ignored --test-threads=1 on a machine with a display"]
fn the_engine_records_the_art_and_music_a_story_shows() {
    let path = file("shown");
    let dir = path.parent().unwrap().to_path_buf();
    let (mut rl, thread) = raylib::init().size(64, 64).title("seen art").build();
    rl.set_trace_log(TraceLogLevel::LOG_WARNING);
    let story = StoryVm::from_source(
        "scene a:\n  background archive_office\n  music archive\n  show mary tired at center\n  \"A line.\"\n",
    );
    let mut manager = ScreenStateManager::with_story(
        &mut rl,
        &thread,
        ScreenState::Playing,
        Box::new(PlayingFactory),
        dir.clone(),
        story,
    )
    .unwrap();
    manager.world.saves = Saves::new(&dir, "Test");
    manager.world.persistent.load(&path);

    for _ in 0..3 {
        manager.update(&mut rl, &thread);
    }
    manager.world.persistent.save();

    let seen = open(&path);
    for key in [
        background_key("archive_office"),
        character_key("mary", "tired"),
        music_key("archive"),
    ] {
        assert!(seen.has_seen(&key), "{key} was not recorded");
    }
}
