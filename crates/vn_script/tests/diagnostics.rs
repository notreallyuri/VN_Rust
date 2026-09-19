use vn_script::{Node, Severity, Value, compile_source, parse_program, tokenize};

fn errors(source: &str) -> Vec<(usize, String)> {
    compile_source(source)
        .diagnostics
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| (d.line, d.message))
        .collect()
}

fn one_error(source: &str) -> (usize, String) {
    let found = errors(source);
    assert_eq!(found.len(), 1, "expected one error, got {:?}", found);
    found.into_iter().next().unwrap()
}

fn in_scene(line: &str) -> String {
    format!("scene start:\n  {}\n", line)
}

fn condition_error(condition: &str) -> String {
    let (line, message) = one_error(&in_scene(&format!("if {}:\n    \"x\"", condition)));
    assert_eq!(line, 2);
    message
}

#[test]
fn a_valid_file_has_no_diagnostics() {
    let source = include_str!("fixtures/all_features.story");
    assert_eq!(compile_source(source).diagnostics, []);
}

#[test]
fn errors_do_not_stop_parsing() {
    let found =
        errors("scene start:\n  show mary\n  \"fine\"\n  jump\n  set x = \n  mary \"also fine\"\n");
    assert_eq!(
        found.iter().map(|(line, _)| *line).collect::<Vec<_>>(),
        [2, 4, 5]
    );
}

#[test]
fn narration_ending_in_a_colon_is_narration() {
    let (tokens, _) = tokenize("scene s:\n  \"She said:\"\n  \"Pick one\":\n");
    assert_eq!(tokens[1].kind, vn_script::TokenKind::Narration);
    assert_eq!(tokens[2].kind, vn_script::TokenKind::ChoiceOption);
}

#[test]
fn stray_choice_option() {
    assert_eq!(
        one_error("scene start:\n  \"Agree\":\n    \"You nod.\"\n  \"after\"\n"),
        (
            2,
            "choice option outside a `choice:` block (narration doesn't end with `:`)".into()
        )
    );
}

#[test]
fn stray_else() {
    assert_eq!(
        one_error("scene start:\n  \"a\"\n  else:\n    \"b\"\n  \"c\"\n"),
        (3, "`else:` without a matching `if` above it".into())
    );
}

#[test]
fn empty_if_body_does_not_swallow_siblings() {
    let source = "scene start:\n  if a == 1:\n  \"sibling\"\n";
    assert_eq!(
        one_error(source),
        (2, "`if` needs an indented block below it".into())
    );

    let (scenes, _) = parse_program(&tokenize(source).0);
    let Node::Scene { body, .. } = &scenes[0].node else {
        panic!("expected a scene");
    };
    assert_eq!(body.len(), 2);
    assert!(matches!(&body[0].node, Node::If { then_branch, .. } if then_branch.is_empty()));
}

#[test]
fn empty_else_body() {
    assert_eq!(
        one_error("scene start:\n  if a == 1:\n    \"x\"\n  else:\n  \"sibling\"\n"),
        (4, "`else:` needs an indented block below it".into())
    );
}

#[test]
fn else_if_is_not_supported() {
    assert_eq!(
        one_error("scene start:\n  if a == 1:\n    \"x\"\n  else if a == 2:\n    \"y\"\n"),
        (
            4,
            "write `else:` on its own line (for `else if`, put an `if` inside `else:`)".into()
        )
    );
}

#[test]
fn empty_option_body() {
    assert_eq!(
        one_error("scene start:\n  choice:\n    \"A\":\n    \"B\":\n      \"b\"\n"),
        (3, "choice option needs an indented block below it".into())
    );
}

#[test]
fn empty_option_text() {
    assert_eq!(
        one_error("scene start:\n  choice:\n    \"  \":\n      \"b\"\n"),
        (3, "choice option text is empty".into())
    );
}

#[test]
fn choice_without_options() {
    assert_eq!(
        one_error("scene start:\n  choice:\n  \"after\"\n"),
        (2, "`choice:` needs at least one option below it".into())
    );
}

#[test]
fn non_option_inside_choice() {
    assert_eq!(
        one_error("scene start:\n  choice:\n    \"A\":\n      \"a\"\n    \"not an option\"\n"),
        (
            5,
            "expected a choice option: quoted text ending with `:`, like `\"Agree\":`".into()
        )
    );
}

#[test]
fn misaligned_option() {
    assert_eq!(
        one_error("scene start:\n  choice:\n    \"A\":\n      \"a\"\n     \"B\":\n      \"b\"\n"),
        (5, "unexpected indentation in `choice:` block".into())
    );
}

#[test]
fn unknown_choice_modifier() {
    assert_eq!(
        one_error("scene start:\n  choice forever:\n    \"A\":\n      \"a\"\n"),
        (
            2,
            "unknown choice modifier `forever` (expected `choice:` or `choice final:`)".into()
        )
    );
}

#[test]
fn show_needs_an_image() {
    assert_eq!(
        one_error(&in_scene("show mary")),
        (2, "`show mary` needs an image: `show mary <image>`".into())
    );
    assert_eq!(
        one_error(&in_scene("show")),
        (
            2,
            "`show` needs a character and an image: `show <character> <image>`".into()
        )
    );
    assert_eq!(
        one_error(&in_scene("show mary happy at the_moon")),
        (
            2,
            "unknown position `the_moon` (expected far_left, left, center, right, far_right)"
                .into()
        )
    );
    assert_eq!(
        one_error(&in_scene("show mary happy at")),
        (
            2,
            "`at` needs a position: far_left, left, center, right, far_right".into()
        )
    );
    assert_eq!(
        one_error(&in_scene("show mary happy near left")),
        (
            2,
            "`show` takes a character, an image and optionally `at <position>` and `with <transition>`; unexpected `near left`"
                .into()
        )
    );
}

#[test]
fn statements_with_wrong_arguments() {
    let cases = [
        (
            "remove",
            "`remove` takes one character: `remove <character>`",
        ),
        (
            "remove a b",
            "`remove` takes one character: `remove <character>`",
        ),
        (
            "clear mary",
            "`clear` takes no arguments (use `remove <character>` for one character)",
        ),
        ("commit now", "`commit` takes no arguments"),
        ("jump", "`jump` takes one scene: `jump <scene>`"),
        ("call", "`call` needs a command: `call <command> [args...]`"),
        ("set x", "expected `set <variable> = <value>`"),
        ("set x =", "expected a value"),
        ("add x", "expected `add <variable> += <integer>` or `-=`"),
        ("add x += lots", "`add` needs an integer amount, got `lots`"),
        ("add x -= -2147483648", "`add` amount is out of range"),
    ];

    for (line, message) in cases {
        assert_eq!(one_error(&in_scene(line)), (2, message.into()), "{}", line);
    }
}

#[test]
fn identifiers_are_checked() {
    let rule = "use lowercase letters, digits and `_`, not starting with a digit";
    let cases = [
        ("show Mary happy", "character id `Mary`"),
        ("show mary Happy", "image id `Happy`"),
        ("remove Mary", "character id `Mary`"),
        ("jump Next", "scene id `Next`"),
        ("call GiveItem", "command name `GiveItem`"),
        ("set 2fast = true", "variable name `2fast`"),
        ("set route = Good", "value `Good`"),
        ("Mary \"Hi\"", "character id `Mary`"),
        ("{Name} \"Hi\"", "variable name `Name`"),
    ];

    for (line, what) in cases {
        assert_eq!(
            one_error(&in_scene(line)),
            (2, format!("invalid {}: {}", what, rule)),
            "{}",
            line
        );
    }

    assert_eq!(
        one_error("scene Start:\n  \"x\"\n"),
        (1, format!("invalid scene id `Start`: {}", rule))
    );
}

#[test]
fn keyword_named_characters() {
    assert_eq!(
        one_error(&in_scene("show \"hi\"")),
        (2, "`show` is a keyword and can't be a character id".into())
    );
    assert_eq!(
        one_error(&in_scene("show clear happy")),
        (2, "`clear` is a keyword and can't be a character id".into())
    );
}

#[test]
fn string_escapes() {
    let program = compile_source(
        "scene start:\n  mary \"She said \\\"hi\\\" \\\\ left\"\n  set name = \"a \\\"b\\\"\"\n  choice:\n    \"Say \\\"yes\\\"\":\n      \"ok\"\n",
    );
    assert_eq!(program.diagnostics, []);

    let texts: Vec<String> = program
        .instructions
        .iter()
        .filter_map(|i| match i {
            vn_script::Instruction::Say { text, .. } => Some(text.clone()),
            vn_script::Instruction::Set { value, .. } => Some(value.to_string()),
            vn_script::Instruction::Choice { options } => Some(options[0].0.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(
        texts,
        ["She said \"hi\" \\ left", "a \"b\"", "Say \"yes\"", "ok"]
    );
}

#[test]
fn escapes_in_conditions() {
    assert_eq!(
        vn_script::parse_condition(r#"name == "a \"b\"""#, 1),
        Ok(vn_script::Condition::Test {
            var_id: "name".into(),
            op: vn_script::Comparison::Equal,
            value: Value::String("a \"b\"".into()),
        })
    );
}

#[test]
fn bad_strings() {
    assert_eq!(
        one_error(&in_scene("\"oops")),
        (2, "unterminated string (missing the closing `\"`)".into())
    );
    assert_eq!(
        one_error(&in_scene(r#""a \n b""#)),
        (
            2,
            r#"unknown escape `\n` (only `\"` and `\\` are allowed)"#.into()
        )
    );
    assert_eq!(
        one_error(&in_scene(r#"mary "hi" there"#)),
        (2, "unexpected `there` after the closing quote".into())
    );
    assert_eq!(
        one_error(&in_scene(r#"set name = "a" b"#)),
        (2, "unexpected `b` after the closing quote".into())
    );
}

#[test]
fn line_that_is_not_dialogue() {
    assert_eq!(
        one_error(&in_scene("mary waves")),
        (
            2,
            "expected dialogue (`<character> \"<text>\"`) or a keyword, found `mary waves`".into()
        )
    );
}

#[test]
fn tabs_in_indentation() {
    assert_eq!(
        one_error("scene start:\n\t\"x\"\n"),
        (2, "tabs aren't allowed in indentation; use spaces".into())
    );
}

#[test]
fn unexpected_indentation() {
    assert_eq!(
        one_error("scene start:\n  \"a\"\n    \"b\"\n    \"c\"\n  \"d\"\n"),
        (3, "unexpected indentation".into())
    );
}

#[test]
fn lines_outside_a_scene() {
    assert_eq!(
        one_error("\"orphan\"\nscene start:\n  \"x\"\n"),
        (
            1,
            "this line is outside a scene; start the file with `scene <id>:`".into()
        )
    );
}

#[test]
fn indented_scene() {
    assert_eq!(
        one_error("scene a:\n  \"x\"\n  scene b:\n    \"y\"\n"),
        (
            3,
            "`scene` can't be indented (scenes can't be nested)".into()
        )
    );
}

#[test]
fn scene_header_needs_a_colon() {
    assert_eq!(
        one_error("scene start\n  \"x\"\n"),
        (1, "`scene <id>` must end with `:`".into())
    );
}

#[test]
fn empty_scene_is_a_warning() {
    let program = compile_source("scene a:\nscene b:\n  \"x\"\n");
    assert_eq!(program.diagnostics.len(), 1);
    assert_eq!(program.diagnostics[0].severity, Severity::Warning);
    assert_eq!(program.diagnostics[0].message, "scene 'a' is empty");
}

#[test]
fn condition_errors() {
    let cases = [
        ("met_mary > true", "`>` only compares integers, got `true`"),
        (r#"a < "b""#, "`<` only compares integers, got `\"b\"`"),
        (
            "met_mary",
            "expected a comparison (== != < > <= >=) after `met_mary`",
        ),
        ("a = 1", "use `==` to compare, not `=`"),
        (
            "a == 1 b == 2",
            "expected `&&`, `||` or the end of the condition, got `b`",
        ),
        (
            r#"a == "oops"#,
            "unterminated string (missing the closing `\"`)",
        ),
        (
            "a == 99999999999",
            "integer `99999999999` is out of range (i32)",
        ),
        (
            "a == 1 &&",
            "expected a variable at the end of the condition",
        ),
        ("a == 1 $", "unexpected `$` in condition"),
    ];

    for (condition, message) in cases {
        assert_eq!(condition_error(condition), message, "{}", condition);
    }

    assert_eq!(
        condition_error("Affection == 1"),
        "invalid variable name `Affection`: use lowercase letters, digits and `_`, not starting with a digit"
    );
}

#[test]
fn if_needs_a_colon_and_a_condition() {
    assert_eq!(
        one_error("scene start:\n  if a == 1\n    \"x\"\n"),
        (2, "`if <condition>` must end with `:`".into())
    );
    assert_eq!(
        one_error("scene start:\n  if:\n    \"x\"\n"),
        (2, "`if` without a condition".into())
    );
}

#[test]
fn errors_inside_a_broken_block_are_still_reported() {
    assert_eq!(
        errors("scene start:\n  if a = 1:\n    show mary\n"),
        [
            (2, "use `==` to compare, not `=`".into()),
            (3, "`show mary` needs an image: `show mary <image>`".into()),
        ]
    );
}

#[test]
fn a_tab_does_not_hide_the_rest_of_the_scene() {
    assert_eq!(
        errors("scene start:\n  \"a\"\n\t\"b\"\n  show mary\n"),
        [
            (3, "tabs aren't allowed in indentation; use spaces".into()),
            (4, "`show mary` needs an image: `show mary <image>`".into()),
        ]
    );
}

#[test]
fn indented_lines_between_scenes() {
    assert_eq!(
        errors(
            "scene a:\n  \"x\"\n\nscene b:\n  \"y\"\n    \"z\"\n   \"w\"\nscene c:\n  show mary\n"
        ),
        [
            (6, "unexpected indentation".into()),
            (9, "`show mary` needs an image: `show mary <image>`".into()),
        ]
    );
}

#[test]
fn dedented_lines_after_a_scene_body() {
    assert_eq!(
        errors("scene a:\n    \"x\"\n  \"y\"\n  \"z\"\nscene b:\n  show mary\n"),
        [
            (3, "unexpected indentation".into()),
            (6, "`show mary` needs an image: `show mary <image>`".into()),
        ]
    );
}

#[test]
fn diagnostics_carry_their_file() {
    let program = vn_script::compile_sources([
        ("a.story", "scene a:\n  show mary\n"),
        (
            "b.story",
            "scene b:\n  \"ok\"\nscene a:\n  \"dup\"\n  jump nowhere\n",
        ),
    ]);

    let shown: Vec<String> = vn_script::Schema::default()
        .validate(&program)
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        shown,
        [
            "a.story:2: error: `show mary` needs an image: `show mary <image>`",
            "b.story:3: error: scene 'a' is already defined at a.story:1; this one is ignored",
        ]
    );

    assert_eq!(program.file(program.scenes["b"]), Some("b.story"));
    assert_eq!(program.line(program.scenes["b"]), 2);
}

#[test]
fn same_file_duplicates_name_the_line() {
    let program =
        vn_script::compile_sources([("a.story", "scene a:\n  \"x\"\nscene a:\n  \"y\"\n")]);
    assert_eq!(
        program.diagnostics[0].to_string(),
        "a.story:3: error: scene 'a' is already defined at line 1; this one is ignored"
    );
}

#[test]
fn transition_errors() {
    assert_eq!(
        one_error(&in_scene("show mary happy with")),
        (
            2,
            "`with` needs a transition: dissolve, fade, slide_left, slide_right".into()
        )
    );
    assert_eq!(
        one_error(&in_scene("background hall with fdae")),
        (
            2,
            "unknown transition `fdae` (expected dissolve, fade, slide_left, slide_right); did you mean 'fade'?"
                .into()
        )
    );
    assert_eq!(
        one_error(&in_scene("remove mary with dissolve slow")),
        (
            2,
            "transition length must be a number of seconds between 0 and 30, got `slow`".into()
        )
    );
    assert_eq!(
        one_error(&in_scene("clear with dissolve 1 2")),
        (
            2,
            "`with` takes a transition and optionally its length in seconds; unexpected `2`".into()
        )
    );
    assert_eq!(
        one_error(&in_scene("clear hugo with fade")),
        (
            2,
            "`clear` takes no arguments (use `remove <character>` for one character)".into()
        )
    );
}

#[test]
fn audio_statements() {
    assert_eq!(
        one_error(&in_scene("music")),
        (
            2,
            "`music` needs a track: `music <track>` or `music none`".into()
        )
    );
    assert_eq!(
        one_error(&in_scene("music hall loud")),
        (2, "`music` takes one track; unexpected `loud`".into())
    );
    assert_eq!(
        one_error(&in_scene("music Hall")),
        (
            2,
            "invalid music track `Hall`: use lowercase letters, digits and `_`, not starting with a digit"
                .into()
        )
    );
    assert_eq!(
        one_error(&in_scene("sound")),
        (2, "`sound` needs a sound: `sound <id>`".into())
    );
    assert_eq!(
        one_error(&in_scene("sound knock twice")),
        (2, "`sound` takes one sound; unexpected `twice`".into())
    );
    assert_eq!(
        one_error(&in_scene("music \"hi\"")),
        (2, "`music` is a keyword and can't be a character id".into())
    );
}

#[test]
fn background_statements() {
    assert_eq!(
        one_error(&in_scene("background")),
        (
            2,
            "`background` needs an image: `background <image>` or `background none`".into()
        )
    );
    assert_eq!(
        one_error(&in_scene("background hall night")),
        (2, "`background` takes one image; unexpected `night`".into())
    );
    assert_eq!(
        one_error(&in_scene("background Hall")),
        (
            2,
            "invalid background id `Hall`: use lowercase letters, digits and `_`, not starting with a digit"
                .into()
        )
    );
    assert_eq!(
        one_error(&in_scene("background \"hi\"")),
        (
            2,
            "`background` is a keyword and can't be a character id".into()
        )
    );
}
