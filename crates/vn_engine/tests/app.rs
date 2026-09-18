use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use vn_engine::script::{CommandSig, ParamKind, Severity, StoryVm};
use vn_engine::{
    AppError, Character, Characters, FromArgs, GameContext, ScreenState, Value, VariableDef, VnApp,
};

struct Project(PathBuf);

impl Project {
    fn new(story: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "vn_engine_app_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("story")).unwrap();
        fs::write(root.join("story/main.story"), story).unwrap();
        Self(root)
    }

    fn add_story(&self, name: &str, story: &str) {
        let path = self.0.join("story").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, story).unwrap();
    }

    fn add_art(&self, character: &str, image: &str) {
        let dir = self.0.join("characters").join(character);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("{}.png", image)), b"").unwrap();
    }

    fn app(&self) -> VnApp {
        VnApp::new("Test")
            .assets(&self.0)
            .character("mary", Character::new("Mary").images(["neutral"]))
            .variable("affection", VariableDef::int(0))
            .variable("player_name", VariableDef::string("Reader"))
            .command("give_item", give_item)
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn give_item(_: &mut GameContext, _: (String, Option<u32>)) -> Option<ScreenState> {
    None
}

#[test]
fn a_valid_story_passes() {
    let project = Project::new(
        "scene start:\n  show mary neutral\n  add affection += 1\n  mary \"Hi, {player_name}.\"\n  call give_item letter 2\n",
    );
    project.add_art("mary", "neutral");

    let (story, warnings) = project.app().check().unwrap();
    assert!(warnings.is_empty(), "{:?}", warnings);
    assert_eq!(story.variable("affection"), Some(&Value::Int(0)));
}

#[test]
fn script_errors_stop_the_app_with_every_error_listed() {
    let project = Project::new(
        "scene start:\n  set afection = 1\n  marie \"Hi\"\n  call give_item letter many\n  call give_itm x\n",
    );

    let error = project.app().check().unwrap_err();
    let AppError::Script { errors, .. } = &error else {
        panic!("expected script errors, got {}", error);
    };

    let lines: Vec<usize> = errors.iter().map(|e| e.line).collect();
    assert_eq!(lines, [2, 3, 4, 5]);

    let message = error.to_string();
    assert!(message.contains("has 4 errors:"), "{}", message);
    assert!(
        message.contains("main.story:2: error: unknown variable 'afection'"),
        "{}",
        message
    );
    assert!(
        message.contains(
            "main.story:4: error: `call give_item` argument 2 should be a non-negative integer, got `many`"
        ),
        "{}",
        message
    );
}

#[test]
fn missing_art_is_a_warning() {
    let project = Project::new("scene start:\n  show mary neutral\n  show mary neutral\n  \"x\"\n");

    let (_, warnings) = project.app().check().unwrap();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].severity, Severity::Warning);
    assert_eq!(warnings[0].line, 2);
    assert!(
        warnings[0]
            .message
            .contains("missing characters/mary/neutral.png")
    );

    let (_, quiet) = project.app().warn_missing_art(false).check().unwrap();
    assert!(quiet.is_empty());
}

#[test]
fn missing_story_dir() {
    let project = Project::new("");
    let error = project.app().story_dir("nope").check().unwrap_err();
    assert!(matches!(error, AppError::Story { .. }));
}

#[test]
fn story_dir_without_stories() {
    let project = Project::new("");
    fs::remove_file(project.0.join("story/main.story")).unwrap();
    fs::write(project.0.join("story/notes.txt"), "not a story").unwrap();

    let error = project.app().check().unwrap_err();
    assert!(
        error.to_string().contains("no .story files in"),
        "{}",
        error
    );
}

#[test]
fn every_story_file_is_loaded() {
    let project = Project::new("scene start:\n  \"one\"\n  jump two\n");
    project.add_story("b/two.story", "scene two:\n  \"two\"\n  jump three\n");
    project.add_story("c.story", "scene three:\n  \"three\"\n");

    let (mut story, _) = project.app().check().unwrap();
    assert_eq!(story.program().files.len(), 3);
    assert_eq!(story.program().scene_order, ["two", "three", "start"]);

    story.set_entry_scene("start").unwrap();
    let mut lines = Vec::new();
    while let vn_engine::script::Event::Say { text, .. } = story.advance_until_blocking() {
        lines.push(text);
    }
    assert_eq!(lines, ["one", "two", "three"]);
}

#[test]
fn errors_name_their_file() {
    let project = Project::new("scene start:\n  \"x\"\n  set nope = 1\n");
    project.add_story("other.story", "scene start:\n  \"dup\"\n");

    let Err(AppError::Script { errors, .. }) = project.app().check() else {
        panic!("expected script errors");
    };
    let shown: Vec<String> = errors.iter().map(ToString::to_string).collect();
    let main = project.0.join("story/main.story").display().to_string();
    let other = project.0.join("story/other.story").display().to_string();

    assert_eq!(
        shown,
        [
            format!("{}:3: error: unknown variable 'nope'", main),
            format!(
                "{}:1: error: scene 'start' is already defined at {}:1; this one is ignored",
                other, main
            ),
        ]
    );
}

#[test]
fn entry_scene() {
    let project = Project::new("scene a:\n  \"a\"\nscene b:\n  \"b\"\n");

    let (story, _) = project.app().entry_scene("b").check().unwrap();
    assert_eq!(story.current_scene(), Some("b"));

    let error = project.app().entry_scene("c").check().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("error: entry scene 'c' does not exist"),
        "{}",
        error
    );
}

#[test]
fn registered_commands_expose_their_signature() {
    let schema = Project::new("").app().schema();
    assert_eq!(
        schema.commands["give_item"],
        CommandSig {
            required: vec![ParamKind::Word],
            optional: vec![ParamKind::UInt],
            rest: None,
        }
    );
    assert_eq!(schema.characters["mary"].images, ["neutral"]);
}

#[test]
fn typed_arguments_parse() {
    let args = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    assert_eq!(
        <(String, Option<u32>)>::from_args(&args(&["letter"])),
        Ok(("letter".to_string(), None))
    );
    assert_eq!(
        <(String, Option<u32>)>::from_args(&args(&["letter", "3"])),
        Ok(("letter".to_string(), Some(3)))
    );
    assert!(<(i32, bool)>::from_args(&args(&["x", "true"])).is_err());
    assert_eq!(
        <(i32, bool, f64)>::from_args(&args(&["-4", "false", "2.5"])),
        Ok((-4, false, 2.5))
    );
    assert_eq!(
        <Vec<String>>::from_args(&args(&["a", "b"])),
        Ok(vec!["a".to_string(), "b".to_string()])
    );
}

#[test]
#[should_panic(expected = "required arguments must come before optional ones")]
fn optional_before_required_is_rejected() {
    <(Option<u32>, String)>::signature();
}

#[test]
fn character_display_names_interpolate() {
    let mut characters = Characters::default();
    characters.insert("player", Character::new("{player_name}"));
    characters.insert("mary", Character::new("Mary"));

    let mut story = StoryVm::from_source("scene a:\n  \"x\"\n");
    story
        .set_variable("player_name", Value::String("Yuri".into()))
        .unwrap();

    assert_eq!(characters.display_name("player", &story), "Yuri");
    assert_eq!(characters.display_name("mary", &story), "Mary");
    assert_eq!(characters.display_name("unknown", &story), "unknown");
}

#[test]
fn parse_errors_are_reported_with_the_file_and_line() {
    let project = Project::new("scene start:\n  show mary\n  if affection = 1:\n    \"x\"\n");

    let Err(error) = project.app().check() else {
        panic!("expected parse errors");
    };
    let AppError::Script { errors, .. } = &error else {
        panic!("expected a script error, got {:?}", error);
    };

    let lines: Vec<usize> = errors.iter().map(|e| e.line).collect();
    assert_eq!(lines, [2, 3]);
    assert!(
        error
            .to_string()
            .contains("main.story:2: error: `show mary` needs an image")
    );
}
