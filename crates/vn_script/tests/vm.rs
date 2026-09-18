use vn_script::{Event, StoryVm, Value, VmError};

fn say(speaker: Option<&str>, text: &str) -> Event {
    Event::Say {
        speaker: speaker.map(str::to_string),
        text: text.to_string(),
    }
}

fn narration(text: &str) -> Event {
    say(None, text)
}

const TWO_SCENES: &str = r#"
scene first:
  "one"
  jump second

scene second:
  show mary neutral
  choice:
    "Left":
      "went left"
    "Right":
      "went right"
  mary "after"
"#;

#[test]
fn starts_at_the_first_scene() {
    let mut vm = StoryVm::from_source(TWO_SCENES);

    assert_eq!(vm.current_scene(), Some("first"));
    assert_eq!(vm.advance(), narration("one"));
}

#[test]
fn jump_enters_the_target_scene() {
    let mut vm = StoryVm::from_source(TWO_SCENES);
    vm.advance();

    assert_eq!(
        vm.advance(),
        Event::Show {
            character: "mary".into(),
            image: "neutral".into(),
        }
    );
    assert_eq!(vm.current_scene(), Some("second"));
    assert_eq!(
        vm.active_characters().get("mary").map(String::as_str),
        Some("neutral")
    );
}

#[test]
fn choice_exits_land_in_the_same_scene() {
    for (index, branch) in [(0, "went left"), (1, "went right")] {
        let mut vm = StoryVm::from_source(TWO_SCENES);
        vm.advance_until_blocking();

        assert_eq!(
            vm.advance_until_blocking(),
            Event::Choice {
                options: vec!["Left".into(), "Right".into()],
            }
        );

        vm.choose(index).unwrap();
        assert_eq!(vm.advance(), narration(branch));
        assert_eq!(vm.advance(), say(Some("mary"), "after"));
        assert_eq!(vm.advance(), Event::End);
    }
}

#[test]
fn choice_repeats_until_answered() {
    let mut vm = StoryVm::from_source(TWO_SCENES);
    vm.advance_until_blocking();

    let choice = vm.advance_until_blocking();
    assert_eq!(vm.advance(), choice);
    assert_eq!(vm.advance(), choice);
}

#[test]
fn choose_rejects_bad_input() {
    let mut vm = StoryVm::from_source(TWO_SCENES);
    assert_eq!(vm.choose(0), Err(VmError::NoChoicePending));

    vm.advance_until_blocking();
    vm.advance_until_blocking();
    assert_eq!(
        vm.choose(2),
        Err(VmError::ChoiceOutOfRange {
            index: 2,
            options: 2
        })
    );
}

#[test]
fn scene_without_jump_ends_the_story() {
    let mut vm = StoryVm::from_source(
        r#"
scene a:
  "only line"

scene b:
  "never reached"
"#,
    );

    assert_eq!(vm.advance(), narration("only line"));
    assert_eq!(vm.advance(), Event::End);
    assert_eq!(vm.advance(), Event::End);
}

#[test]
fn jump_to_unknown_scene_ends_the_story() {
    let mut vm = StoryVm::from_source(
        r#"
scene a:
  jump nowhere
"#,
    );

    assert_eq!(vm.advance(), Event::End);
}

#[test]
fn jump_loop_without_dialogue_ends_the_story() {
    let mut vm = StoryVm::from_source(
        r#"
scene a:
  jump b

scene b:
  jump a
"#,
    );

    assert_eq!(vm.advance(), Event::End);
}

#[test]
fn reset_restarts_with_empty_state() {
    let mut vm = StoryVm::from_source(TWO_SCENES);
    vm.advance_until_blocking();
    vm.advance_until_blocking();
    assert!(!vm.active_characters().is_empty());

    vm.reset();

    assert_eq!(vm.current_scene(), Some("first"));
    assert!(vm.active_characters().is_empty());
    assert_eq!(vm.advance(), narration("one"));
}

#[test]
fn start_at_enters_a_given_scene() {
    let mut vm = StoryVm::from_source(TWO_SCENES);

    assert_eq!(
        vm.start_at("missing"),
        Err(VmError::UnknownScene("missing".into()))
    );

    vm.start_at("second").unwrap();
    assert_eq!(vm.current_scene(), Some("second"));
}

#[test]
fn empty_source_ends_immediately() {
    let mut vm = StoryVm::from_source("");

    assert_eq!(vm.current_scene(), None);
    assert_eq!(vm.advance(), Event::End);
}

#[test]
fn the_example_plays_through_every_chapter() {
    let mut vm = StoryVm::from_dir("../../examples/god_is_watching/assets/story").unwrap();
    assert!(vm.program().unknown_jump_targets().is_empty());

    let mut scenes = std::collections::BTreeSet::new();
    let mut lines = 0;
    loop {
        scenes.extend(vm.current_scene().map(str::to_string));
        match vm.advance_until_blocking() {
            Event::End => break,
            Event::Choice { .. } => vm.choose(0).unwrap(),
            _ => lines += 1,
        }
        assert!(lines < 10_000, "the story never ends");
    }

    for first in ["mary_start", "moriarty_start", "post_start"] {
        assert!(scenes.contains(first), "never reached {}", first);
    }
    assert_eq!(vm.current_scene(), Some("post_last"));
}

#[test]
fn story_files_in_a_directory() {
    let dir = std::env::temp_dir().join(format!("vn_script_dir_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("nested")).unwrap();
    std::fs::write(dir.join("b.story"), "scene b:\n  \"b\"\n").unwrap();
    std::fs::write(dir.join("nested/a.story"), "scene a:\n  \"a\"\n").unwrap();
    std::fs::write(dir.join("a.story"), "scene first:\n  jump b\n").unwrap();
    std::fs::write(dir.join("notes.txt"), "ignored").unwrap();

    let files = vn_script::story_files(&dir).unwrap();
    let names: Vec<_> = files
        .iter()
        .map(|f| f.strip_prefix(&dir).unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["a.story", "b.story", "nested/a.story"]);

    let mut vm = StoryVm::from_dir(&dir).unwrap();
    assert_eq!(vm.entry_scene(), Some("first"));
    assert_eq!(
        vm.advance_until_blocking(),
        Event::Say {
            speaker: None,
            text: "b".into()
        }
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

fn branch_taken(setup: &str, condition: &str) -> bool {
    let source = format!(
        "scene start:\n{}\n  if {}:\n    \"then\"\n  else:\n    \"else\"\n",
        setup, condition
    );
    let mut vm = StoryVm::from_source(&source);

    match vm.advance_until_blocking() {
        Event::Say { text, .. } => text == "then",
        other => panic!("expected a line, got {:?}", other),
    }
}

#[test]
fn integer_comparisons() {
    let setup = "  set affection = 3";

    assert!(branch_taken(setup, "affection == 3"));
    assert!(branch_taken(setup, "affection != 2"));
    assert!(branch_taken(setup, "affection >= 3"));
    assert!(branch_taken(setup, "affection <= 3"));
    assert!(branch_taken(setup, "affection > 2"));
    assert!(branch_taken(setup, "affection < 4"));
    assert!(!branch_taken(setup, "affection > 3"));
    assert!(!branch_taken(setup, "affection < 3"));
}

#[test]
fn bool_and_enum_comparisons() {
    let setup = "  set met_mary = true\n  set route = good";

    assert!(branch_taken(setup, "met_mary == true"));
    assert!(branch_taken(setup, "met_mary != false"));
    assert!(!branch_taken(setup, "met_mary == false"));
    assert!(branch_taken(setup, "route == good"));
    assert!(branch_taken(setup, "route != bad"));
    assert!(!branch_taken(setup, "route == bad"));
}

#[test]
fn and_or() {
    let setup = "  set a = 1\n  set b = false";

    assert!(!branch_taken(setup, "a == 1 && b == true"));
    assert!(branch_taken(setup, "a == 1 || b == true"));
    assert!(branch_taken(setup, "a == 2 || a == 1 && b == false"));
    assert!(!branch_taken(setup, "a == 2 || a == 1 && b == true"));
}

#[test]
fn unset_variables_use_the_literal_type_default() {
    assert!(branch_taken("", "met_mary == false"));
    assert!(!branch_taken("", "met_mary == true"));
    assert!(branch_taken("", "affection == 0"));
    assert!(branch_taken("", "affection < 1"));
    assert!(!branch_taken("", "route == good"));
    assert!(branch_taken("", "route != good"));
}

#[test]
fn type_mismatch_is_false() {
    assert!(!branch_taken("  set a = true", "a == 1"));
    assert!(!branch_taken("  set a = true", "a != 1"));
}

#[test]
fn set_and_add_update_variables() {
    let mut vm = StoryVm::from_source(
        "scene start:\n  set met_mary = true\n  add affection += 5\n  add affection -= 2\n  \"done\"\n",
    );
    vm.advance_until_blocking();

    assert_eq!(vm.variable("met_mary"), Some(&Value::Bool(true)));
    assert_eq!(vm.variable("affection"), Some(&Value::Int(3)));
}

#[test]
fn add_leaves_non_integers_alone() {
    let mut vm = StoryVm::from_source(
        "scene start:\n  set met_mary = true\n  add met_mary += 1\n  \"done\"\n",
    );
    vm.advance_until_blocking();

    assert_eq!(vm.variable("met_mary"), Some(&Value::Bool(true)));
}

#[test]
fn fixture_plays_through() {
    let mut vm = StoryVm::from_source(include_str!("fixtures/all_features.story"));
    let mut lines = Vec::new();
    let mut calls = Vec::new();
    let mut speakers = Vec::new();

    loop {
        match vm.advance() {
            Event::Say { speaker, text } => {
                speakers.extend(speaker);
                lines.push(text);
            }
            Event::Call { command, .. } => calls.push(command),
            Event::Choice { .. } => vm.choose(1).unwrap(),
            Event::End => break,
            _ => {}
        }
    }

    assert!(lines.contains(&"You forgot.".to_string()));
    assert!(lines.contains(&"Something shifts.".to_string()));
    assert!(lines.contains(&"The sign says \"Closed\".".to_string()));
    assert!(lines.contains(&"Nice to meet you, Yuri.".to_string()));
    assert!(speakers.contains(&"Yuri".to_string()));
    assert!(lines.contains(&"You shake your head.".to_string()));
    assert_eq!(lines.last().unwrap(), "Things just got darker.");
    assert_eq!(calls, ["give_item", "unlock_route", "ask_name"]);
    assert_eq!(vm.variable("affection"), Some(&Value::Int(-1)));
}

#[test]
fn string_comparisons() {
    let setup = "  set player_name = \"Yuri\"";

    assert!(branch_taken(setup, r#"player_name == "Yuri""#));
    assert!(branch_taken(setup, r#"player_name != "Mary""#));
    assert!(!branch_taken(setup, r#"player_name == "yuri""#));
    assert!(branch_taken("", r#"player_name == """#));
    assert!(!branch_taken(setup, "player_name == yuri"));
}

#[test]
fn interpolates_text_speaker_and_choices() {
    let mut vm = StoryVm::from_source(
        r#"
scene start:
  mary "Nice to meet you, {player_name}."
  {player_name} "Nice to meet you, mary."
  choice:
    "Ask {player_name}'s age":
      "ok"
"#,
    );
    vm.set_variable("player_name", Value::String("Yuri".into()))
        .unwrap();

    assert_eq!(vm.advance(), say(Some("mary"), "Nice to meet you, Yuri."));
    assert_eq!(vm.advance(), say(Some("Yuri"), "Nice to meet you, mary."));
    assert_eq!(
        vm.advance(),
        Event::Choice {
            options: vec!["Ask Yuri's age".into()],
        }
    );
}

#[test]
fn interpolation_uses_the_value_at_display_time() {
    let mut vm = StoryVm::from_source(
        "scene start:\n  \"{player_name}\"\n  set player_name = \"Mary\"\n  \"{player_name}\"\n",
    );

    assert_eq!(vm.advance(), narration("{player_name}"));
    assert_eq!(vm.advance(), narration("Mary"));
}

#[test]
fn current_is_the_event_waiting_for_the_player() {
    let mut vm = StoryVm::from_source(TWO_SCENES);
    assert_eq!(vm.current(), None);

    let line = vm.advance_until_blocking();
    assert_eq!(vm.current(), Some(&line));

    let choice = vm.advance_until_blocking();
    assert_eq!(vm.current(), Some(&choice));

    vm.choose(0).unwrap();
    assert_eq!(vm.current(), None);

    vm.advance_until_blocking();
    vm.reset();
    assert_eq!(vm.current(), None);
}

#[test]
fn current_is_cleared_by_non_blocking_events() {
    let mut vm = StoryVm::from_source(TWO_SCENES);

    assert_eq!(vm.advance(), narration("one"));
    assert!(vm.current().is_some());

    assert!(matches!(vm.advance(), Event::Show { .. }));
    assert_eq!(vm.current(), None);
}

fn events_until_blocking(vm: &mut StoryVm) -> Vec<Event> {
    let mut events = Vec::new();
    loop {
        let event = vm.advance();
        let blocking = event.is_blocking();
        events.push(event);
        if blocking {
            return events;
        }
    }
}

fn scene_enter(scene: &str) -> Event {
    Event::SceneEnter {
        scene: scene.into(),
    }
}

const JUMPING: &str =
    "scene a:\n  \"a1\"\n  \"a2\"\n  jump b\nscene b:\n  show mary neutral\n  \"b1\"\n";

#[test]
fn scene_events_are_off_by_default() {
    let mut vm = StoryVm::from_source(JUMPING);
    assert!(
        std::iter::from_fn(|| Some(vm.advance()))
            .take_while(|e| *e != Event::End)
            .all(|e| !matches!(e, Event::SceneEnter { .. }))
    );
}

#[test]
fn scene_events_mark_the_start_and_every_jump() {
    let mut vm = StoryVm::from_source(JUMPING);
    vm.set_scene_events(true);

    assert_eq!(events_until_blocking(&mut vm)[0], scene_enter("a"));
    assert_eq!(events_until_blocking(&mut vm).len(), 1);
    assert_eq!(
        events_until_blocking(&mut vm)[..2],
        [
            scene_enter("b"),
            Event::Show {
                character: "mary".into(),
                image: "neutral".into()
            }
        ]
    );

    vm.reset();
    assert_eq!(events_until_blocking(&mut vm)[0], scene_enter("a"));
}

#[test]
fn jumping_to_the_same_scene_enters_it_again() {
    let mut vm = StoryVm::from_source("scene a:\n  \"x\"\n  jump a\n");
    vm.set_scene_events(true);
    events_until_blocking(&mut vm);
    assert_eq!(events_until_blocking(&mut vm)[0], scene_enter("a"));
}

#[test]
fn restoring_the_exact_position_does_not_enter_the_scene() {
    let mut vm = StoryVm::from_source(JUMPING);
    vm.set_scene_events(true);
    events_until_blocking(&mut vm);
    let snapshot = vm.snapshot();

    let mut restored = StoryVm::from_source(JUMPING);
    restored.set_scene_events(true);
    restored.restore(&snapshot).unwrap();
    assert_eq!(events_until_blocking(&mut restored).len(), 1);

    let edited = JUMPING.replace("\"a2\"", "\"a2 (edited)\"");
    let mut restarted = StoryVm::from_source(&edited);
    restarted.set_scene_events(true);
    restarted.restore(&snapshot).unwrap();
    assert_eq!(events_until_blocking(&mut restarted)[0], scene_enter("a"));
}
