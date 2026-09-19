use vn_script::{
    CharacterDef, CommandSig, Diagnostic, Event, ParamKind, Schema, Severity, StoryVm, Value,
    VariableDef, VmError,
};

fn schema() -> Schema {
    let mut schema = Schema::default();
    schema
        .variables
        .insert("affection".into(), VariableDef::int(0));
    schema
        .variables
        .insert("met_mary".into(), VariableDef::bool(false));
    schema.variables.insert(
        "route".into(),
        VariableDef::enumeration(["good", "bad", "neutral"], "neutral"),
    );
    schema
        .variables
        .insert("player_name".into(), VariableDef::string("Mary"));
    schema.characters.insert(
        "mary".into(),
        CharacterDef {
            name: "Mary".into(),
            images: vec!["neutral".into(), "happy".into()],
        },
    );
    schema.characters.insert(
        "hugo".into(),
        CharacterDef {
            name: "Hugo".into(),
            images: vec![],
        },
    );
    schema.commands.insert(
        "give_item".into(),
        CommandSig {
            required: vec![ParamKind::Word],
            optional: vec![ParamKind::UInt],
            rest: None,
        },
    );
    schema
}

fn check(source: &str) -> Vec<Diagnostic> {
    let mut vm = StoryVm::from_source(source);
    vm.set_schema(schema());
    vm.validate()
}

fn messages(source: &str) -> Vec<(usize, String)> {
    check(source)
        .into_iter()
        .map(|d| (d.line, d.message))
        .collect()
}

#[test]
fn a_valid_story_has_no_diagnostics() {
    let source = r#"
scene start:
  show mary happy
  show hugo anything
  set met_mary = true
  set route = good
  set player_name = "Yuri"
  add affection += 2
  if affection >= 2 && route != bad || met_mary == false:
    mary "Hello, {player_name}."
  {player_name} "Hi, Mary."
  choice:
    "Tell {player_name}'s story":
      call give_item letter
      call give_item letter 3
  remove hugo
  jump start
"#;

    assert_eq!(check(source), []);
}

#[test]
fn unknown_names_are_errors_with_lines() {
    let source = r#"
scene start:
  set afection = 1
  maryy "Hi"
  show hugh neutral
  "Hello, {plyer}."
  call give_itm stick
  jump nowhere
"#;

    assert_eq!(
        messages(source),
        [
            (
                3,
                "unknown variable 'afection'; did you mean 'affection'?".into()
            ),
            (
                4,
                "speaker: unknown character 'maryy'; did you mean 'mary'?".into()
            ),
            (
                5,
                "`show`: unknown character 'hugh'; did you mean 'hugo'?".into()
            ),
            (6, "unknown variable 'plyer'".into()),
            (
                7,
                "unknown command 'give_itm'; did you mean 'give_item'?".into()
            ),
            (8, "`jump nowhere`: no scene with that name".into()),
        ]
    );
    assert!(check(source).iter().all(|d| d.severity == Severity::Error));
}

#[test]
fn type_errors() {
    let source = r#"
scene start:
  set affection = true
  set met_mary = 3
  set route = great
  set player_name = yuri
  add met_mary += 1
  if route == 3:
    "x"
  if player_name == 3:
    "y"
"#;

    let found = messages(source);
    assert_eq!(found.len(), 7, "{:#?}", found);
    assert_eq!(
        found[0],
        (3, "`set affection`: expected int, got `true`".into())
    );
    assert_eq!(
        found[1],
        (4, "`set met_mary`: expected bool, got `3`".into())
    );
    assert_eq!(
        found[2],
        (
            5,
            "`set route`: 'great' is not one of good | bad | neutral".into()
        )
    );
    assert_eq!(
        found[3],
        (
            6,
            "`set player_name`: expected a string, got the bare word `yuri` (write \"yuri\")"
                .into()
        )
    );
    assert_eq!(
        found[4],
        (7, "`add met_mary`: variable is bool, not int".into())
    );
    assert_eq!(found[5].0, 8);
    assert_eq!(found[6].0, 10);
}

#[test]
fn undeclared_images() {
    assert_eq!(
        messages("scene start:\n  show mary angry\n"),
        [(
            2,
            "`show mary angry`: 'mary' has no image 'angry' (images: neutral, happy)".into()
        )]
    );
}

#[test]
fn command_arguments() {
    let found =
        messages("scene start:\n  call give_item\n  call give_item a b\n  call give_item a 1 2\n");

    assert_eq!(
        found,
        [
            (
                2,
                "`call give_item` takes 1 to 2 arguments, got 0 (usage: call give_item <word> [uint])"
                    .into()
            ),
            (
                3,
                "`call give_item` argument 2 should be a non-negative integer, got `b`".into()
            ),
            (
                4,
                "`call give_item` takes 1 to 2 arguments, got 3 (usage: call give_item <word> [uint])"
                    .into()
            ),
        ]
    );
}

#[test]
fn variadic_commands() {
    let sig = CommandSig {
        required: vec![ParamKind::Word],
        optional: vec![],
        rest: Some(ParamKind::Int),
    };

    assert!(sig.check("sum", &["x".into()]).is_ok());
    assert!(
        sig.check("sum", &["x".into(), "1".into(), "-2".into()])
            .is_ok()
    );
    assert!(sig.check("sum", &["x".into(), "one".into()]).is_err());
    assert!(sig.check("sum", &[]).is_err());
}

#[test]
fn empty_registries_check_nothing() {
    let vm = StoryVm::from_source(
        "scene start:\n  set anything = 1\n  whoever \"Hi {x}\"\n  call whatever 1 2 3\n",
    );
    assert_eq!(vm.validate(), []);
}

#[test]
fn compile_errors_are_diagnostics() {
    let vm = StoryVm::from_source("scene a:\n  \"one\"\nscene a:\n  \"two\"\n");
    assert_eq!(
        vm.validate(),
        [Diagnostic::error(
            3,
            "scene 'a' is already defined at line 1; this one is ignored"
        )]
    );
}

#[test]
fn diagnostics_inside_branches_point_at_their_line() {
    let found = messages(
        "scene start:\n  choice:\n    \"A\":\n      set nope = 1\n  if affection > 0:\n    \"x\"\n  else:\n    set nada = 2\n",
    );
    assert_eq!(
        found,
        [
            (4, "unknown variable 'nope'".into()),
            (8, "unknown variable 'nada'".into())
        ]
    );
}

#[test]
fn variables_start_at_their_defaults() {
    let mut vm = StoryVm::from_source("scene start:\n  \"{player_name} {affection} {route}\"\n");
    vm.set_schema(schema());

    assert_eq!(vm.variable("met_mary"), Some(&Value::Bool(false)));
    assert_eq!(
        vm.advance(),
        Event::Say {
            speaker: None,
            text: "Mary 0 neutral".into()
        }
    );
}

#[test]
fn set_variable_is_type_checked() {
    let mut vm = StoryVm::from_source("scene start:\n  \"x\"\n");
    vm.set_schema(schema());

    assert!(
        vm.set_variable("player_name", Value::String("Yuri".into()))
            .is_ok()
    );
    assert_eq!(
        vm.set_variable("nobody", Value::Int(1)),
        Err(VmError::UnknownVariable("nobody".into()))
    );
    assert!(matches!(
        vm.set_variable("affection", Value::Bool(true)),
        Err(VmError::TypeMismatch { .. })
    ));
}

#[test]
fn entry_scene() {
    let mut vm = StoryVm::from_source("scene a:\n  \"a\"\nscene b:\n  \"b\"\n");

    assert_eq!(
        vm.set_entry_scene("missing"),
        Err(VmError::UnknownScene("missing".into()))
    );
    vm.set_entry_scene("b").unwrap();
    assert_eq!(vm.current_scene(), Some("b"));

    vm.advance();
    vm.reset();
    assert_eq!(
        vm.advance(),
        Event::Say {
            speaker: None,
            text: "b".into()
        }
    );
}

#[test]
fn loading_an_old_save_fills_in_new_variables() {
    let source = "scene start:\n  \"x\"\n";
    let old = StoryVm::from_source(source).snapshot();

    let mut vm = StoryVm::from_source(source);
    vm.set_schema(schema());
    vm.restore(&old).unwrap();

    assert_eq!(vm.variable("affection"), Some(&Value::Int(0)));
    assert_eq!(
        vm.variable("player_name"),
        Some(&Value::String("Mary".into()))
    );
}

#[test]
#[should_panic(expected = "enum default 'maybe' is not one of")]
fn enum_default_must_be_a_member() {
    VariableDef::enumeration(["yes", "no"], "maybe");
}

#[test]
fn every_example_story_is_valid_without_registries() {
    let vm = StoryVm::from_dir("../../examples/god_is_watching/assets/story").unwrap();
    assert_eq!(vm.program().files.len(), 6);
    assert_eq!(vm.validate(), []);
}

#[test]
fn schema_files_round_trip() {
    let mut file = vn_script::SchemaFile::new("Game", "story", schema());
    file.entry_scene = Some("start".into());

    let json = file.to_json();
    assert_eq!(vn_script::SchemaFile::from_json(&json), Ok(file.clone()));
    assert!(json.contains("\"story_dir\": \"story\""));
    assert!(json.contains("\"give_item\""));

    let path = std::env::temp_dir().join(format!("vn_schema_{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);
    assert!(file.write(&path).unwrap());
    assert!(!file.write(&path).unwrap());
    assert_eq!(vn_script::SchemaFile::read(&path).unwrap(), file);
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn schema_files_can_leave_registries_out() {
    let file = vn_script::SchemaFile::from_json(
        r#"{ "format_version": 1, "game": "G", "story_dir": "story", "characters": { "mary": { "name": "Mary" } } }"#,
    )
    .unwrap();
    assert!(file.schema.variables.is_empty());
    assert_eq!(file.schema.characters["mary"].images, Vec::<String>::new());
    assert_eq!(file.entry_scene, None);
}

#[test]
fn newer_schema_files_are_rejected() {
    let error = vn_script::SchemaFile::from_json(
        r#"{ "format_version": 2, "game": "G", "story_dir": "story" }"#,
    )
    .unwrap_err();
    assert_eq!(
        error,
        "schema format 2 is newer than this tool supports (1)"
    );
}

#[test]
fn prepare_applies_the_schema_and_the_entry_scene() {
    let mut vm = StoryVm::from_source("scene a:\n  \"a\"\nscene b:\n  set nope = 1\n");
    assert_eq!(
        vm.prepare(schema(), Some("missing")),
        [
            Diagnostic::error(0, "entry scene 'missing' does not exist"),
            Diagnostic::error(4, "unknown variable 'nope'"),
        ]
    );

    assert_eq!(vm.prepare(schema(), Some("b")).len(), 1);
    assert_eq!(vm.entry_scene(), Some("b"));
}
