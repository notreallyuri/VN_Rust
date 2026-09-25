use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

use novn::app::AppError;
use novn::data::rollback::Rollback;
use novn::data::state::GameState;
use novn::game::hot_reload::{StoryLoader, StoryWatcher, swap_story};
use novn::game::script_errors::ScriptErrors;
use novn::script::{Event, RestoreOutcome, Schema, StoryVm, VariableDef};

struct Dir(PathBuf);

impl Dir {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "vn_engine_hot_reload_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("story")).unwrap();
        Self(dir)
    }

    fn write(&self, name: &str, source: &str, age: u64) {
        let path = self.0.join("story").join(name);
        fs::write(&path, source).unwrap();
        let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000 + age);
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(time)
            .unwrap();
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const STORY: &str = "scene a:\n  \"a1\"\n  \"a2\"\n  jump b\nscene b:\n  \"b1\"\n  \"b2\"\n";

#[test]
fn the_watcher_notices_edits_new_files_and_deletions() {
    let dir = Dir::new();
    dir.write("01.story", STORY, 0);
    let mut watcher = StoryWatcher::new(dir.0.join("story"));
    assert!(!watcher.changed());

    dir.write("01.story", STORY, 1);
    assert!(watcher.changed());
    assert!(!watcher.changed(), "reported once");

    dir.write("02.story", "scene c:\n  \"c\"\n", 0);
    assert!(watcher.changed());

    fs::remove_file(dir.0.join("story/02.story")).unwrap();
    assert!(watcher.changed());

    fs::write(dir.0.join("story/notes.txt"), "not a story").unwrap();
    assert!(!watcher.changed(), "other files are ignored");
}

#[test]
fn the_watcher_checks_at_most_twice_a_second() {
    let dir = Dir::new();
    dir.write("01.story", STORY, 0);
    let mut watcher = StoryWatcher::new(dir.0.join("story"));

    assert!(!watcher.poll(10.0));
    dir.write("01.story", STORY, 1);
    assert!(!watcher.poll(10.2));
    assert!(watcher.poll(10.6));
}

fn played(source: &str, lines: usize) -> (StoryVm, Rollback) {
    let mut story = StoryVm::from_source(source);
    let mut rollback = Rollback::default();
    let state = GameState::default();
    for _ in 0..lines {
        story.advance_until_blocking();
        rollback.record(&story, &state);
    }
    (story, rollback)
}

fn line(story: &StoryVm) -> String {
    match story.current() {
        Some(Event::Say { text, .. }) => text.clone(),
        other => format!("{:?}", other),
    }
}

#[test]
fn editing_another_scene_keeps_the_line_and_the_history() {
    let (mut story, mut rollback) = played(STORY, 4);
    assert_eq!(line(&story), "b2");

    let edited = STORY.replace("\"a1\"", "\"a1 (edited)\"");
    let outcome = swap_story(&mut story, StoryVm::from_source(&edited), &mut rollback).unwrap();

    assert_eq!(outcome, RestoreOutcome::Exact);
    assert_eq!(line(&story), "b2");
    assert_eq!(
        rollback.steps_back(),
        1,
        "only the unchanged scene's history is kept"
    );
}

#[test]
fn editing_the_current_scene_restarts_it() {
    let (mut story, mut rollback) = played(STORY, 4);

    let edited = STORY.replace("\"b2\"", "\"b2 (edited)\"");
    let outcome = swap_story(&mut story, StoryVm::from_source(&edited), &mut rollback).unwrap();

    assert_eq!(
        outcome,
        RestoreOutcome::SceneRestarted { scene: "b".into() }
    );
    assert_eq!(story.current_scene(), Some("b"));
    assert_eq!(story.current(), None);
    assert!(!rollback.can_go_back());
    assert_eq!(
        story.advance_until_blocking(),
        Event::Say {
            speaker: None,
            text: "b1".into()
        }
    );
}

#[test]
fn deleting_the_current_scene_keeps_the_old_story() {
    let (mut story, mut rollback) = played(STORY, 4);

    let without_b = "scene a:\n  \"a1\"\n";
    assert!(swap_story(&mut story, StoryVm::from_source(without_b), &mut rollback).is_err());
    assert_eq!(line(&story), "b2");
    assert!(rollback.can_go_back());
}

#[test]
fn the_loader_validates_like_startup() {
    let dir = Dir::new();
    dir.write("01.story", "scene start:\n  set mood = 1\n  \"x\"\n", 0);
    let mut schema = Schema::default();
    schema.variables.insert("mood".into(), VariableDef::int(0));

    let loader = StoryLoader {
        #[cfg(feature = "character-visuals")]
        visuals: Default::default(),
        assets: dir.0.clone().into(),
        story_dir: "story".into(),
        schema,
        entry_scene: Some("start".into()),
        warn_missing_art: false,
    };
    let (story, warnings) = loader.load().unwrap();
    assert!(warnings.is_empty());
    assert_eq!(story.entry_scene(), Some("start"));

    dir.write("01.story", "scene start:\n  set nope = 1\n", 1);
    let Err(AppError::Script { errors, .. }) = loader.load() else {
        panic!("expected script errors");
    };
    assert_eq!(errors[0].message, "unknown variable 'nope'");
}

#[test]
fn failed_reloads_list_their_errors_relative_to_the_story_dir() {
    let dir = Dir::new();
    dir.write("01.story", "scene a:\n  remoe hugo\n  jump bb\n", 0);
    let loader = StoryLoader {
        #[cfg(feature = "character-visuals")]
        visuals: Default::default(),
        assets: dir.0.clone().into(),
        story_dir: PathBuf::from("story"),
        schema: Schema::default(),
        entry_scene: None,
        warn_missing_art: false,
    };

    let error = loader.load().unwrap_err();
    let errors = ScriptErrors::from_error(&error, &loader.path());

    assert_eq!(
        errors.title,
        "Story not reloaded: 2 errors (still running the previous version)"
    );
    assert_eq!(errors.lines.len(), 2);
    assert!(
        errors.lines[0].starts_with("01.story:2: error: expected dialogue"),
        "{}",
        errors.lines[0]
    );
    assert!(errors.lines[0].ends_with("did you mean `remove`?"));
    assert_eq!(
        errors.lines[1],
        "01.story:3: error: `jump bb`: no scene with that name"
    );
    assert!(!errors.collapsed);
}

#[test]
fn a_missing_story_dir_is_one_line() {
    let dir = Dir::new();
    fs::remove_dir_all(dir.0.join("story")).unwrap();
    let loader = StoryLoader {
        #[cfg(feature = "character-visuals")]
        visuals: Default::default(),
        assets: dir.0.clone().into(),
        story_dir: PathBuf::from("story"),
        schema: Schema::default(),
        entry_scene: None,
        warn_missing_art: false,
    };

    let errors = ScriptErrors::from_error(&loader.load().unwrap_err(), &loader.path());
    assert_eq!(errors.lines.len(), 1);
    assert!(errors.lines[0].starts_with("could not load the story"));
    assert!(errors.title.contains("1 error "));
}
