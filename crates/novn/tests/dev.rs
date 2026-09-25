use novn::dev::scene_jump::{Panel, SceneJump, Step};
use novn::input::navigation::NavInput;
use novn::script::{Schema, StoryVm, Value, VariableDef};

const STORY: &str = r#"scene start:
  "The first line."
  jump archive

scene archive:
  if trust >= 3:
    "You are trusted."
  "The archive."

scene ending:
  "The end."
"#;

fn story() -> StoryVm {
    let mut schema = Schema::default();
    schema.variables.insert("trust".into(), VariableDef::int(0));
    schema
        .variables
        .insert("met_mary".into(), VariableDef::bool(false));
    schema.variables.insert(
        "route".into(),
        VariableDef::enumeration(["good", "bad", "neutral"], "good"),
    );
    schema
        .variables
        .insert("player_name".into(), VariableDef::string("Reader"));
    let mut vm = StoryVm::from_source(STORY);
    vm.set_schema(schema);
    vm
}

fn press(nav: impl FnOnce(&mut NavInput)) -> NavInput {
    let mut input = NavInput::default();
    nav(&mut input);
    input
}

#[test]
fn every_scene_is_listed_in_story_order() {
    let jump = SceneJump::from_story(&story());
    let ids: Vec<_> = jump.scenes.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(ids, ["start", "archive", "ending"]);
    assert_eq!(jump.scenes[1].place, "line 5");
}

#[test]
fn variables_are_listed_by_name_with_their_current_values() {
    let jump = SceneJump::from_story(&story());
    let ids: Vec<_> = jump.variables.iter().map(|row| row.id.as_str()).collect();
    assert_eq!(ids, ["met_mary", "player_name", "route", "trust"]);
    let trust = jump.variables.iter().find(|row| row.id == "trust").unwrap();
    assert_eq!(trust.value, Value::Int(0));
    assert!(!trust.changed);
}

#[test]
fn the_selection_starts_on_the_scene_being_played() {
    let mut vm = story();
    vm.start_at("ending").unwrap();
    let jump = SceneJump::from_story(&vm);
    assert_eq!(jump.selected_scene(), Some("ending"));
}

#[test]
fn arrows_move_through_scenes_and_stop_at_the_ends() {
    let mut jump = SceneJump::from_story(&story());
    jump.step(&press(|n| n.up = true));
    assert_eq!(jump.scene, 0);
    for _ in 0..5 {
        jump.step(&press(|n| n.down = true));
    }
    assert_eq!(jump.selected_scene(), Some("ending"));
}

#[test]
fn tab_switches_panels_and_right_enters_the_variables() {
    let mut jump = SceneJump::from_story(&story());
    jump.step(&press(|n| n.next = true));
    assert_eq!(jump.panel, Panel::Variables);
    jump.step(&press(|n| n.next = true));
    assert_eq!(jump.panel, Panel::Scenes);
    jump.step(&press(|n| n.right = true));
    assert_eq!(jump.panel, Panel::Variables);
}

#[test]
fn each_kind_of_variable_is_adjusted_its_own_way() {
    let mut jump = SceneJump::from_story(&story());
    let at = |jump: &SceneJump, id: &str| jump.variables.iter().position(|r| r.id == id).unwrap();

    jump.variable = at(&jump, "met_mary");
    jump.adjust(1);
    assert_eq!(jump.variables[jump.variable].value, Value::Bool(true));

    jump.variable = at(&jump, "trust");
    jump.adjust(1);
    jump.adjust(1);
    jump.adjust(1);
    assert_eq!(jump.variables[jump.variable].value, Value::Int(3));
    jump.adjust(-5);
    assert_eq!(jump.variables[jump.variable].value, Value::Int(-2));

    jump.variable = at(&jump, "route");
    jump.adjust(-1);
    assert_eq!(
        jump.variables[jump.variable].value,
        Value::Enum("neutral".into())
    );
    jump.adjust(1);
    assert_eq!(
        jump.variables[jump.variable].value,
        Value::Enum("good".into())
    );
}

#[test]
fn a_string_is_shown_but_not_edited() {
    let mut jump = SceneJump::from_story(&story());
    jump.variable = jump
        .variables
        .iter()
        .position(|r| r.id == "player_name")
        .unwrap();
    assert!(!jump.variables[jump.variable].editable());
    jump.adjust(1);
    assert_eq!(
        jump.variables[jump.variable].value,
        Value::String("Reader".into())
    );
    assert!(!jump.variables[jump.variable].changed);
}

#[test]
fn a_jump_lands_on_the_scene_with_the_values_that_were_set() {
    let mut vm = story();
    let mut jump = SceneJump::from_story(&vm);
    jump.scene = 1;
    jump.variable = jump.variables.iter().position(|r| r.id == "trust").unwrap();
    for _ in 0..3 {
        jump.adjust(1);
    }

    jump.apply(&mut vm).unwrap();
    assert_eq!(vm.current_scene(), Some("archive"));
    assert_eq!(vm.variable("trust"), Some(&Value::Int(3)));

    let said: Vec<String> = std::iter::from_fn(|| match vm.advance_until_blocking() {
        novn::script::Event::Say { text, .. } => Some(text),
        _ => None,
    })
    .collect();
    assert_eq!(said.first().map(String::as_str), Some("You are trusted."));
}

#[test]
fn values_carry_over_from_the_game_being_played() {
    let mut vm = story();
    vm.set_variable("trust", Value::Int(7)).unwrap();
    let jump = SceneJump::from_story(&vm);
    let trust = jump.variables.iter().find(|r| r.id == "trust").unwrap();
    assert_eq!(
        trust.value,
        Value::Int(7),
        "a jump starts from what you have now"
    );
}

#[test]
fn accept_jumps_and_back_closes() {
    let mut jump = SceneJump::from_story(&story());
    assert_eq!(jump.step(&press(|n| n.accept = true)), Step::Jump);
    assert_eq!(jump.step(&press(|n| n.back = true)), Step::Close);
    assert_eq!(jump.step(&NavInput::default()), Step::Stay);
}

#[test]
fn a_story_with_no_scenes_does_not_jump() {
    let vm = StoryVm::from_source("");
    let mut jump = SceneJump::from_story(&vm);
    assert_eq!(jump.step(&press(|n| n.accept = true)), Step::Stay);
}

#[test]
fn the_list_does_not_move_when_the_selection_is_already_visible() {
    use novn::dev::scene_jump::followed;
    assert_eq!(
        followed(2, 6, 30, 20),
        2,
        "a click inside the view keeps the view"
    );
    assert_eq!(
        followed(2, 21, 30, 20),
        2,
        "the last visible row keeps it too"
    );
}

#[test]
fn the_list_scrolls_just_enough_to_follow_the_selection() {
    use novn::dev::scene_jump::followed;
    assert_eq!(
        followed(2, 1, 30, 20),
        1,
        "one above the view scrolls up by one"
    );
    assert_eq!(
        followed(2, 22, 30, 20),
        3,
        "one below the view scrolls down by one"
    );
    assert_eq!(followed(0, 29, 30, 20), 10, "never past the end");
    assert_eq!(followed(9, 3, 5, 20), 0, "a short list never scrolls");
}

#[test]
fn a_list_opens_with_the_selection_in_the_middle() {
    use novn::dev::scene_jump::centred;
    assert_eq!(centred(12, 30, 20), 2);
    assert_eq!(centred(0, 30, 20), 0);
    assert_eq!(centred(29, 30, 20), 10);
}
