use novn_script::{Event, StoryVm, Value, VmError};

fn option(text: &str, index: usize) -> novn_script::ChoiceOption {
    novn_script::ChoiceOption::new(text, index)
}

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
            position: None,
            transition: None,
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
                options: vec![option("Left", 0), option("Right", 1)],
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

const EXAMPLE: &str = "../../examples/god_is_watching/assets";

fn example() -> StoryVm {
    let mut vm = StoryVm::from_dir(format!("{}/story", EXAMPLE)).unwrap();
    let schema = novn_script::SchemaFile::read(format!("{}/schema.json", EXAMPLE)).unwrap();
    assert_eq!(vm.prepare(schema.schema, schema.entry_scene.as_deref()), []);
    vm
}

fn play_to_the_end(vm: &mut StoryVm, last_choice: usize) -> std::collections::BTreeSet<String> {
    let mut scenes = std::collections::BTreeSet::new();
    let mut lines = 0;
    loop {
        scenes.extend(vm.current_scene().map(str::to_string));
        match vm.advance_until_blocking() {
            Event::End => return scenes,
            Event::Choice { options } => {
                let last = options
                    .iter()
                    .any(|o| o.text.starts_with("\"A boy with a blessing"));
                let taken = match last {
                    true => last_choice,
                    false => options
                        .iter()
                        .position(|o| o.enabled)
                        .expect("a choice offers something to take"),
                };
                vm.choose(options[taken].index).unwrap();
            }
            _ => lines += 1,
        }
        assert!(lines < 10_000, "the story never ends");
    }
}

#[test]
fn the_example_plays_through_every_chapter() {
    let mut vm = example();
    assert!(vm.program().unknown_jump_targets().is_empty());

    let scenes = play_to_the_end(&mut vm, 0);
    for first in [
        "archive_start",
        "box_start",
        "notebook_start",
        "reports_start",
        "ilde_start",
        "report_start",
    ] {
        assert!(scenes.contains(first), "never reached {}", first);
    }
}

#[test]
fn every_ending_of_the_example_is_reachable() {
    for (choice, ending) in [(0, "report"), (1, "silence"), (2, "keeper")] {
        let mut vm = example();
        let scenes = play_to_the_end(&mut vm, choice);
        assert!(scenes.contains(&format!("ending_{}", ending)), "{}", ending);
        assert_eq!(vm.variable("ending"), Some(&Value::Enum(ending.into())));
        assert_eq!(vm.variable("recognized_clara"), Some(&Value::Bool(true)));
        assert!(matches!(vm.variable("trust"), Some(Value::Int(n)) if *n >= 2));
    }
}

#[test]
fn the_hidden_ending_needs_the_clues() {
    let mut vm = example();
    let mut offered = 0;
    loop {
        match vm.advance_until_blocking() {
            Event::End => break,
            Event::Choice { options } => {
                offered = options.len();
                let at = options
                    .iter()
                    .position(|o| {
                        o.text.starts_with("Leave it")
                            || o.text.starts_with("Let the margin")
                            || o.text.starts_with("Thank her")
                    })
                    .or_else(|| options.iter().position(|o| o.enabled))
                    .expect("a choice offers something to take");
                vm.choose(options[at].index).unwrap();
            }
            _ => {}
        }
    }
    assert_eq!(
        offered, 2,
        "without the clues the final report has two options"
    );
    assert_eq!(vm.variable("recognized_clara"), Some(&Value::Bool(false)));
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

    let files = novn_script::story_files(&dir).unwrap();
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
            Event::Choice { options } => {
                let last = options.last().expect("a choice has options");
                vm.choose(last.index).unwrap()
            }
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
    assert!(lines.contains(&"Things just got darker.".to_string()));
    assert!(lines.contains(&"You leave it where it was.".to_string()));
    assert_eq!(lines.last().unwrap(), "Her voice does not echo.");
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
            options: vec![option("Ask Yuri's age", 0)],
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
                image: "neutral".into(),
                position: None,
                transition: None,
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

const STAGE: &str = r#"
scene a:
  background hall
  show mary tired at left
  show hugo neutral
  "one"
  show mary happy
  "two"
  remove mary
  show mary tired
  "three"
  show hugo neutral at far_right
  clear
  "four"
  background none
  "five"
"#;

fn next_line(vm: &mut StoryVm) {
    assert!(matches!(vm.advance_until_blocking(), Event::Say { .. }));
}

#[test]
fn positions_and_backgrounds() {
    use novn_script::Position;
    let mut vm = StoryVm::from_source(STAGE);

    next_line(&mut vm);
    assert_eq!(vm.background(), Some("hall"));
    assert_eq!(vm.position("mary"), Some(Position::Left));
    assert_eq!(vm.position("hugo"), None);

    next_line(&mut vm);
    assert_eq!(
        vm.position("mary"),
        Some(Position::Left),
        "a new expression keeps the spot"
    );

    next_line(&mut vm);
    assert_eq!(vm.position("mary"), None, "remove forgets the spot");

    next_line(&mut vm);
    assert!(vm.active_characters().is_empty());
    assert_eq!(vm.position("hugo"), None, "clear forgets every spot");
    assert_eq!(vm.background(), Some("hall"), "clear keeps the background");

    next_line(&mut vm);
    assert_eq!(vm.background(), None);

    vm.reset();
    next_line(&mut vm);
    vm.reset();
    assert_eq!(vm.background(), None);
    assert_eq!(vm.position("mary"), None);
}

#[test]
fn show_and_background_events() {
    let mut vm = StoryVm::from_source(STAGE);
    assert_eq!(
        vm.advance(),
        Event::Background {
            image: Some("hall".into()),
            transition: None,
        }
    );
    assert_eq!(
        vm.advance(),
        Event::Show {
            character: "mary".into(),
            image: "tired".into(),
            position: Some(novn_script::Position::Left),
            transition: None,
        }
    );
}

#[test]
fn snapshots_keep_positions_and_the_background() {
    let mut vm = StoryVm::from_source(STAGE);
    next_line(&mut vm);
    let json = serde_json::to_string(&vm.snapshot()).unwrap();

    let mut restored = StoryVm::from_source(STAGE);
    restored
        .restore(&serde_json::from_str(&json).unwrap())
        .unwrap();
    assert_eq!(restored.background(), Some("hall"));
    assert_eq!(restored.position("mary"), Some(novn_script::Position::Left));

    let mut old: serde_json::Value = serde_json::from_str(&json).unwrap();
    let object = old.as_object_mut().unwrap();
    object.remove("positions");
    object.remove("background");
    let old: novn_script::StorySnapshot = serde_json::from_value(old).unwrap();
    assert!(old.positions.is_empty());
    assert_eq!(old.background, None);
}

const SOUNDTRACK: &str = r#"
scene a:
  music hall_theme
  sound knock
  "one"
  music storm
  "two"
  music none
  "three"
"#;

#[test]
fn music_is_state_and_sounds_are_events() {
    let mut vm = StoryVm::from_source(SOUNDTRACK);
    assert_eq!(vm.music(), None);
    assert_eq!(
        vm.advance(),
        Event::Music {
            track: Some("hall_theme".into())
        }
    );
    assert_eq!(vm.music(), Some("hall_theme"));
    assert_eq!(vm.advance(), Event::Sound { id: "knock".into() });
    assert!(matches!(vm.advance(), Event::Say { .. }));

    next_line(&mut vm);
    assert_eq!(vm.music(), Some("storm"));
    next_line(&mut vm);
    assert_eq!(vm.music(), None);

    vm.reset();
    next_line(&mut vm);
    assert_eq!(vm.music(), Some("hall_theme"));
    vm.reset();
    assert_eq!(vm.music(), None);
}

#[test]
fn snapshots_keep_the_music() {
    let mut vm = StoryVm::from_source(SOUNDTRACK);
    next_line(&mut vm);
    next_line(&mut vm);
    let snapshot = vm.snapshot();
    assert_eq!(snapshot.music.as_deref(), Some("storm"));

    let mut restored = StoryVm::from_source(SOUNDTRACK);
    restored.restore(&snapshot).unwrap();
    assert_eq!(restored.music(), Some("storm"));

    let mut silent = StoryVm::from_source(SOUNDTRACK);
    next_line(&mut silent);
    next_line(&mut silent);
    next_line(&mut silent);
    let json = serde_json::to_value(silent.snapshot()).unwrap();
    assert!(json.get("music").is_none(), "no music, no field");
    let old: novn_script::StorySnapshot = serde_json::from_value(json).unwrap();
    assert_eq!(old.music, None);
}

const TRANSITIONS: &str = r#"
scene a:
  background hall with fade
  show mary happy at left with dissolve 0.25
  show hugo neutral
  "one"
  remove mary with slide_left
  clear with dissolve
  background none with dissolve
  "two"
"#;

#[test]
fn transitions_ride_on_the_next_event() {
    use novn_script::{Transition, TransitionKind::*};
    let mut vm = StoryVm::from_source(TRANSITIONS);

    let fade = Some(Transition::new(Fade));
    assert_eq!(
        vm.advance(),
        Event::Background {
            image: Some("hall".into()),
            transition: fade,
        }
    );
    match vm.advance() {
        Event::Show { transition, .. } => {
            let transition = transition.unwrap();
            assert_eq!(transition.kind, Dissolve);
            assert_eq!(transition.seconds(), 0.25);
        }
        other => panic!("{:?}", other),
    }
    assert!(
        matches!(
            vm.advance(),
            Event::Show {
                transition: None,
                ..
            }
        ),
        "a transition applies to one statement"
    );
    assert!(matches!(vm.advance(), Event::Say { .. }));
    assert_eq!(
        vm.advance(),
        Event::Hide {
            character: "mary".into(),
            transition: Some(Transition::new(SlideLeft)),
        }
    );
    assert_eq!(
        vm.advance(),
        Event::Clear {
            transition: Some(Transition::new(Dissolve)),
        }
    );
    assert!(matches!(
        vm.advance(),
        Event::Background {
            image: None,
            transition: Some(_)
        }
    ));
    assert_eq!(Transition::new(Fade).seconds(), 1.0);
    assert_eq!(Transition::new(SlideRight).to_string(), "slide_right");
}

#[test]
fn scenes_without_transitions_compile_as_before() {
    let plain = StoryVm::from_source("scene a:\n  show mary happy\n  \"hi\"\n");
    let with = StoryVm::from_source("scene a:\n  show mary happy with dissolve\n  \"hi\"\n");
    assert_eq!(plain.program().instructions.len(), 3);
    assert_eq!(
        with.program().instructions.len(),
        4,
        "one extra `With` instruction"
    );
    assert!(matches!(
        with.program().instructions[0],
        novn_script::Instruction::With(_)
    ));
}

#[test]
fn voice_clips_are_events_and_lines_have_stable_keys() {
    let source = "scene a:\n  voice mary_1\n  mary \"Hello.\"\n  \"Hello.\"\n  choice:\n    \"Go\":\n      \"x\"\n";
    let mut vm = StoryVm::from_source(source);
    assert_eq!(vm.line_key(), None);
    assert_eq!(
        vm.advance(),
        Event::Voice {
            id: "mary_1".into()
        }
    );
    assert!(matches!(vm.advance(), Event::Say { .. }));
    let first = vm.line_key().expect("a line");

    let mut again = StoryVm::from_source(source);
    again.advance_until_blocking();
    assert_eq!(again.line_key(), Some(first), "the same line, the same key");

    let mut restored = StoryVm::from_source(source);
    restored.restore(&vm.snapshot()).unwrap();
    assert_eq!(restored.line_key(), Some(first), "and after a load");

    vm.advance();
    let narration = vm.line_key().unwrap();
    assert_ne!(narration, first, "the speaker is part of the key");
    vm.advance();
    assert_eq!(vm.line_key(), None, "choices have no key");
}

#[test]
fn a_show_without_a_position_serializes_as_before() {
    let show = novn_script::Instruction::Show {
        char_id: "mary".into(),
        img_id: "tired".into(),
        position: None,
    };
    assert_eq!(
        serde_json::to_string(&show).unwrap(),
        r#"{"Show":{"char_id":"mary","img_id":"tired"}}"#
    );
}

const GATED: &str = r#"scene start:
  choice:
    "Open the door" when has_key == true "The door is locked":
      "It opens."
    "Look through the window" unless curtains == true:
      "You see a table."
    "Turn back":
      "You leave."
"#;

fn gated() -> StoryVm {
    let mut vm = StoryVm::from_source(GATED);
    vm.set_schema(schema_with([
        ("has_key", novn_script::VariableDef::bool(false)),
        ("curtains", novn_script::VariableDef::bool(false)),
    ]));
    vm
}

fn schema_with<const N: usize>(
    variables: [(&str, novn_script::VariableDef); N],
) -> novn_script::Schema {
    let mut schema = novn_script::Schema::default();
    for (name, def) in variables {
        schema.variables.insert(name.to_string(), def);
    }
    schema
}

fn offered(vm: &mut StoryVm) -> Vec<novn_script::ChoiceOption> {
    match vm.advance_until_blocking() {
        Event::Choice { options } => options,
        other => panic!("expected a choice, got {:?}", other),
    }
}

#[test]
fn an_option_with_a_reason_is_offered_but_disabled() {
    let mut vm = gated();
    let options = offered(&mut vm);

    assert_eq!(options[0].text, "Open the door");
    assert!(!options[0].enabled);
    assert_eq!(options[0].reason.as_deref(), Some("The door is locked"));
    assert_eq!(vm.choose(0), Err(VmError::ChoiceUnavailable { index: 0 }));
}

#[test]
fn an_option_without_a_reason_is_hidden_until_its_condition_holds() {
    let mut vm = gated();
    vm.set_variable("curtains", Value::Bool(true)).unwrap();
    let options = offered(&mut vm);

    let texts: Vec<&str> = options.iter().map(|o| o.text.as_str()).collect();
    assert_eq!(texts, ["Open the door", "Turn back"]);
    assert_eq!(options[1].index, 2, "the index is the one to choose");

    vm.choose(options[1].index).unwrap();
    assert_eq!(vm.advance(), narration("You leave."));
}

#[test]
fn a_met_condition_leaves_the_option_alone() {
    let mut vm = gated();
    vm.set_variable("has_key", Value::Bool(true)).unwrap();
    let options = offered(&mut vm);

    assert!(options[0].enabled);
    assert_eq!(options[0].reason, None);
    vm.choose(0).unwrap();
    assert_eq!(vm.advance(), narration("It opens."));
}

#[test]
fn a_choice_whose_options_are_all_hidden_is_skipped() {
    let mut vm = StoryVm::from_source(
        "scene start:\n  choice:\n    \"A\" when flag == true:\n      \"a\"\n    \"B\" when flag == true:\n      \"b\"\n  \"after\"\n",
    );
    vm.set_schema(schema_with([(
        "flag",
        novn_script::VariableDef::bool(false),
    )]));

    assert_eq!(vm.advance_until_blocking(), narration("after"));
}

#[test]
fn an_option_carries_its_pictures() {
    let mut vm = StoryVm::from_source(
        "scene start:\n  choice:\n    \"The north road\" image north preview north_view:\n      \"You walk north.\"\n",
    );
    let options = offered(&mut vm);

    assert_eq!(options[0].image.as_deref(), Some("north"));
    assert_eq!(options[0].preview.as_deref(), Some("north_view"));
}

#[test]
fn a_scene_chooses_nvl_mode_in_its_header() {
    let vm = StoryVm::from_source(
        "scene start nvl:\n  \"One.\"\n  jump hall\n\nscene hall:\n  \"Two.\"\n",
    );

    assert_eq!(
        vm.program().scene_mode("start"),
        novn_script::SceneMode::Nvl
    );
    assert_eq!(vm.program().scene_mode("hall"), novn_script::SceneMode::Adv);
    assert!(vm.scene_mode().is_nvl(), "the entry scene is in NVL mode");
}

#[test]
fn the_mode_follows_the_scene_the_story_is_in() {
    let mut vm = StoryVm::from_source(
        "scene start:\n  \"One.\"\n  jump hall\n\nscene hall nvl:\n  \"Two.\"\n",
    );

    vm.advance_until_blocking();
    assert!(!vm.scene_mode().is_nvl());
    vm.advance_until_blocking();
    assert!(vm.scene_mode().is_nvl());
}

#[test]
fn the_line_on_screen_knows_its_file_and_line() {
    let program = novn_script::compile_sources(vec![
        (
            "a.story".to_string(),
            "scene start:\n  \"one\"\n  jump next\n".to_string(),
        ),
        (
            "b.story".to_string(),
            "scene next:\n  mary \"two\"\n  choice:\n    \"Go\":\n      \"three\"\n".to_string(),
        ),
    ]);
    let mut vm = StoryVm::from_program(program);
    assert_eq!(
        vm.current_source(),
        None,
        "nothing is on screen before the first advance"
    );

    vm.advance_until_blocking();
    assert_eq!(vm.current_source(), Some(("a.story", 2)));
    vm.advance_until_blocking();
    assert_eq!(vm.current_source(), Some(("b.story", 2)));
    vm.advance_until_blocking();
    assert_eq!(
        vm.current_source(),
        Some(("b.story", 3)),
        "a waiting choice is its own line"
    );
}
