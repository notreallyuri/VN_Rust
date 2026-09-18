use vn_script::{Event, RestoreOutcome, StoryVm, Value, VmError};

const STORY: &str = r#"
scene intro:
  "one"
  jump middle

scene middle:
  show mary neutral
  set met_mary = true
  mary "two"
  choice:
    "Left":
      "went left"
    "Right":
      "went right"
  "three"
  jump outro

scene outro:
  "four"
"#;

fn narration(text: &str) -> Event {
    Event::Say {
        speaker: None,
        text: text.to_string(),
    }
}

fn play_to(vm: &mut StoryVm, text: &str) {
    loop {
        match vm.advance_until_blocking() {
            Event::Say { text: t, .. } if t == text => return,
            Event::Choice { .. } => vm.choose(0).unwrap(),
            Event::End => panic!("never reached {:?}", text),
            _ => {}
        }
    }
}

#[test]
fn round_trip_mid_scene() {
    let mut vm = StoryVm::from_source(STORY);
    play_to(&mut vm, "two");
    let snapshot = vm.snapshot();

    let mut restored = StoryVm::from_source(STORY);
    assert_eq!(restored.restore(&snapshot), Ok(RestoreOutcome::Exact));

    assert_eq!(restored.current_scene(), Some("middle"));
    assert_eq!(restored.current(), vm.current());
    assert_eq!(restored.variable("met_mary"), Some(&Value::Bool(true)));
    assert_eq!(restored.active_characters(), vm.active_characters());
    assert!(matches!(restored.advance(), Event::Choice { .. }));
}

#[test]
fn round_trip_at_a_choice() {
    let mut vm = StoryVm::from_source(STORY);
    play_to(&mut vm, "two");
    let choice = vm.advance_until_blocking();
    let snapshot = vm.snapshot();

    let mut restored = StoryVm::from_source(STORY);
    restored.restore(&snapshot).unwrap();

    assert_eq!(restored.current(), Some(&choice));
    restored.choose(1).unwrap();
    assert_eq!(restored.advance(), narration("went right"));
}

#[test]
fn snapshot_survives_json() {
    let mut vm = StoryVm::from_source(STORY);
    play_to(&mut vm, "two");
    let snapshot = vm.snapshot();

    let json = serde_json::to_string(&snapshot).unwrap();
    assert_eq!(
        serde_json::from_str::<vn_script::StorySnapshot>(&json).unwrap(),
        snapshot
    );
}

#[test]
fn editing_another_scene_keeps_the_save_exact() {
    let mut vm = StoryVm::from_source(STORY);
    play_to(&mut vm, "two");
    let snapshot = vm.snapshot();

    let edited = STORY.replace(
        "scene intro:\n  \"one\"",
        "scene intro:\n  \"zero\"\n  \"one\"\n  \"one and a half\"",
    );
    let mut restored = StoryVm::from_source(&edited);

    assert_eq!(restored.restore(&snapshot), Ok(RestoreOutcome::Exact));
    assert!(matches!(restored.advance(), Event::Choice { .. }));
}

#[test]
fn editing_the_saved_scene_restarts_it() {
    let mut vm = StoryVm::from_source(STORY);
    play_to(&mut vm, "two");
    let snapshot = vm.snapshot();

    let edited = STORY.replace("mary \"two\"", "mary \"two, rewritten\"");
    let mut restored = StoryVm::from_source(&edited);

    assert_eq!(
        restored.restore(&snapshot),
        Ok(RestoreOutcome::SceneRestarted {
            scene: "middle".into()
        })
    );
    assert_eq!(restored.current(), None);
    assert_eq!(restored.variable("met_mary"), Some(&Value::Bool(true)));
    assert!(matches!(restored.advance(), Event::Show { .. }));
}

#[test]
fn missing_scene_is_an_error_and_leaves_the_vm_untouched() {
    let mut vm = StoryVm::from_source(STORY);
    play_to(&mut vm, "four");
    let snapshot = vm.snapshot();

    let without_outro = STORY.replace("scene outro:\n  \"four\"\n", "");
    let mut other = StoryVm::from_source(&without_outro);
    other.advance();

    assert_eq!(
        other.restore(&snapshot),
        Err(VmError::UnknownScene("outro".into()))
    );
    assert_eq!(other.current_scene(), Some("intro"));
    assert_eq!(other.current(), Some(&narration("one")));
}

#[test]
fn snapshot_of_a_new_story_restores_to_the_start() {
    let snapshot = StoryVm::from_source(STORY).snapshot();
    let mut restored = StoryVm::from_source(STORY);
    restored.advance();

    assert_eq!(restored.restore(&snapshot), Ok(RestoreOutcome::Exact));
    assert_eq!(restored.advance(), narration("one"));
}
