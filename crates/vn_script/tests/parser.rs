use vn_script::{
    Comparison, Compiler, Condition, Diagnostic, Node, Stmt, Value, parse_condition, parse_program,
    tokenize,
};

const ALL_FEATURES: &str = include_str!("fixtures/all_features.story");

fn test(var_id: &str, op: Comparison, value: Value) -> Condition {
    Condition::Test {
        var_id: var_id.to_string(),
        op,
        value,
    }
}

fn parse(source: &str) -> (Vec<Stmt>, Vec<Diagnostic>) {
    let (tokens, mut diagnostics) = tokenize(source);
    let (scenes, parse_diagnostics) = parse_program(&tokens);
    diagnostics.extend(parse_diagnostics);
    (scenes, diagnostics)
}

fn condition(expr: &str) -> Condition {
    parse_condition(expr, 1).unwrap_or_else(|d| panic!("{}", d))
}

fn scene_body(source: &str) -> Vec<Node> {
    let (mut scenes, diagnostics) = parse(source);
    assert_eq!(diagnostics, []);
    match scenes.remove(0).node {
        Node::Scene { body, .. } => body.into_iter().map(|stmt| stmt.node).collect(),
        other => panic!("expected a scene, got {:?}", other),
    }
}

#[test]
fn fixture_parses_and_compiles() {
    let (scenes, diagnostics) = parse(ALL_FEATURES);
    assert_eq!(diagnostics, []);
    assert_eq!(scenes.len(), 2);

    let program = Compiler::new().compile(scenes);
    assert_eq!(program.scene_order, ["start", "second_scene"]);
    assert!(program.unknown_jump_targets().is_empty());
}

#[test]
fn single_comparison() {
    assert_eq!(
        condition("met_mary == true"),
        test("met_mary", Comparison::Equal, Value::Bool(true))
    );
}

#[test]
fn every_operator() {
    let cases = [
        ("==", Comparison::Equal),
        ("!=", Comparison::NotEqual),
        (">=", Comparison::Gte),
        ("<=", Comparison::Lte),
        (">", Comparison::Gt),
        ("<", Comparison::Lt),
    ];

    for (symbol, op) in cases {
        assert_eq!(
            condition(&format!("affection {} 3", symbol)),
            test("affection", op, Value::Int(3)),
            "{}",
            symbol
        );
    }
}

#[test]
fn operators_without_spaces() {
    assert_eq!(
        condition("affection>=-2"),
        test("affection", Comparison::Gte, Value::Int(-2))
    );
}

#[test]
fn and_binds_tighter_than_or() {
    assert_eq!(
        condition("a == 1 || b == 2 && c == true"),
        Condition::Any(vec![
            test("a", Comparison::Equal, Value::Int(1)),
            Condition::All(vec![
                test("b", Comparison::Equal, Value::Int(2)),
                test("c", Comparison::Equal, Value::Bool(true)),
            ]),
        ])
    );
}

#[test]
fn bare_identifiers_are_enum_members() {
    assert_eq!(
        condition("route != good"),
        test("route", Comparison::NotEqual, Value::Enum("good".into()))
    );
}

#[test]
fn set_and_add() {
    let body = scene_body(
        r#"
scene start:
  set met_mary = true
  set route = good
  set count = -4
  add affection += 2
  add affection -= 3
"#,
    );

    assert_eq!(
        body,
        [
            Node::Set {
                var_id: "met_mary".into(),
                value: Value::Bool(true),
            },
            Node::Set {
                var_id: "route".into(),
                value: Value::Enum("good".into()),
            },
            Node::Set {
                var_id: "count".into(),
                value: Value::Int(-4),
            },
            Node::Add {
                var_id: "affection".into(),
                amount: 2,
            },
            Node::Add {
                var_id: "affection".into(),
                amount: -3,
            },
        ]
    );
}

#[test]
fn string_literals() {
    assert_eq!(
        condition(r#"player_name == "Yuri""#),
        test(
            "player_name",
            Comparison::Equal,
            Value::String("Yuri".into())
        )
    );
}

#[test]
fn operators_inside_strings_are_text() {
    assert_eq!(
        condition(r#"a == "x && y || z == 1" && b != """#),
        Condition::All(vec![
            test(
                "a",
                Comparison::Equal,
                Value::String("x && y || z == 1".into())
            ),
            test("b", Comparison::NotEqual, Value::String(String::new())),
        ])
    );
}

#[test]
fn set_string() {
    assert_eq!(
        scene_body("scene start:\n  set player_name = \"Mary Ann = 2\"\n"),
        [Node::Set {
            var_id: "player_name".into(),
            value: Value::String("Mary Ann = 2".into()),
        }]
    );
}

#[test]
fn interpolated_speaker_is_kept_as_written() {
    assert_eq!(
        scene_body("scene start:\n  {player_name} \"Nice to meet you, mary.\"\n"),
        [Node::Dialogue {
            speaker: Some("{player_name}".into()),
            text: "Nice to meet you, mary.".into(),
        }]
    );
}

#[test]
fn commit_and_final_choices() {
    let body = scene_body(
        "scene start:\n  choice final:\n    \"A\":\n      \"a\"\n  commit\n  choice:\n    \"B\":\n      \"b\"\n",
    );

    assert!(matches!(
        body[0],
        Node::ChoiceBlock {
            final_choice: true,
            ..
        }
    ));
    assert_eq!(body[1], Node::Commit);
    assert!(matches!(
        body[2],
        Node::ChoiceBlock {
            final_choice: false,
            ..
        }
    ));
}

#[test]
fn final_choices_compile_a_commit_into_every_option() {
    let (scenes, _) =
        parse("scene start:\n  choice final:\n    \"A\":\n      \"a\"\n    \"B\":\n      \"b\"\n");
    let program = Compiler::new().compile(scenes);

    let commits = program
        .instructions
        .iter()
        .filter(|i| matches!(i, vn_script::Instruction::Commit))
        .count();
    assert_eq!(commits, 2);
}
