use novn_script::{compile_sources, format};

const ALL_FEATURES: &str = include_str!("fixtures/all_features.story");

fn same_story(before: &str, after: &str) {
    let before = compile_sources([("before", before)]);
    let after = compile_sources([("after", after)]);
    assert!(before.diagnostics.iter().all(|d| !d.is_error()));
    assert!(after.diagnostics.iter().all(|d| !d.is_error()));
    assert_eq!(
        format!("{:?}", before.instructions),
        format!("{:?}", after.instructions)
    );
    assert_eq!(before.scene_order, after.scene_order);
}

#[test]
fn indentation_becomes_two_spaces_per_level() {
    let source = "scene start:\n      show mary neutral\n      if seen == true:\n                \"Deep.\"\n";
    assert_eq!(
        format(source),
        "scene start:\n  show mary neutral\n  if seen == true:\n    \"Deep.\"\n"
    );
    same_story(source, &format(source));
}

#[test]
fn runs_of_spaces_between_words_collapse() {
    let source = "scene start:\n  show   mary    neutral   at left   with dissolve   0.5\n";
    assert_eq!(
        format(source),
        "scene start:\n  show mary neutral at left with dissolve 0.5\n"
    );
}

#[test]
fn operators_get_spaces_in_conditions_and_assignments() {
    let source = "scene start:\n  set mood=\"tense\"\n  add trust+=2\n  if trust>=2&&mood==\"tense\":\n    \"Yes.\"\n";
    assert_eq!(
        format(source),
        "scene start:\n  set mood = \"tense\"\n  add trust += 2\n  if trust >= 2 && mood == \"tense\":\n    \"Yes.\"\n"
    );
    same_story(source, &format(source));
}

#[test]
fn text_inside_strings_is_left_alone() {
    let source =
        "scene start:\n  mary \"Two  spaces, a #hash, a colon: and {name}, \\\"quoted\\\" too\"\n";
    assert_eq!(format(source), source);
}

#[test]
fn call_arguments_are_not_treated_as_operators() {
    let source = "scene start:\n  call give_item note=1 2 true\n";
    assert_eq!(format(source), source);
}

#[test]
fn scenes_are_separated_by_one_blank_line() {
    let source = "scene one:\n  \"a\"\nscene two:\n  \"b\"\n\n\n\nscene three:\n  \"c\"\n";
    assert_eq!(
        format(source),
        "scene one:\n  \"a\"\n\nscene two:\n  \"b\"\n\nscene three:\n  \"c\"\n"
    );
    same_story(source, &format(source));
}

#[test]
fn a_comment_above_a_scene_keeps_the_blank_line_above_it() {
    let source = "scene one:\n  \"a\"\n# about two\n# still about two\nscene two:\n  \"b\"\n";
    assert_eq!(
        format(source),
        "scene one:\n  \"a\"\n\n# about two\n# still about two\nscene two:\n  \"b\"\n"
    );
}

#[test]
fn comments_are_indented_like_the_line_below_them() {
    let source = "scene start:\n# about the show\n      show mary neutral\n      if seen == true:\n# about the line\n        \"Deep.\"\n";
    assert_eq!(
        format(source),
        "scene start:\n  # about the show\n  show mary neutral\n  if seen == true:\n    # about the line\n    \"Deep.\"\n"
    );
}

#[test]
fn a_trailing_comment_keeps_the_indentation_of_the_line_above() {
    let source = "scene start:\n  \"a\"\n  # done\n";
    assert_eq!(format(source), source);
}

#[test]
fn blank_lines_collapse_and_the_file_ends_with_one_newline() {
    let source = "\n\nscene start:\n  \"a\"\n\n\n  \"b\"   \n\n\n";
    assert_eq!(format(source), "scene start:\n  \"a\"\n\n  \"b\"\n");
}

#[test]
fn an_empty_file_stays_empty() {
    assert_eq!(format(""), "");
    assert_eq!(format("\n\n  \n"), "");
}

#[test]
fn the_fixture_survives_formatting_and_is_stable() {
    let once = format(ALL_FEATURES);
    assert_eq!(once, format(&once));
    same_story(ALL_FEATURES, &once);
}

#[test]
fn option_conditions_get_spaces_around_their_operators() {
    let source = "scene start:\n  choice:\n    \"Open\"when has_key==true\"The door is locked\":\n      \"a\"\n";
    assert_eq!(
        format(source),
        "scene start:\n  choice:\n    \"Open\" when has_key == true \"The door is locked\":\n      \"a\"\n"
    );
    same_story(source, &format(source));
}

#[test]
fn option_pictures_keep_their_order() {
    let source = "scene start:\n  choice:\n    \"North\"   image north    preview north_view:\n      \"a\"\n";
    assert_eq!(
        format(source),
        "scene start:\n  choice:\n    \"North\" image north preview north_view:\n      \"a\"\n"
    );
    same_story(source, &format(source));
}

#[test]
fn a_scene_mode_survives_formatting() {
    let source = "scene start    nvl:\n      \"One.\"\n";
    assert_eq!(format(source), "scene start nvl:\n  \"One.\"\n");
    same_story(source, &format(source));
}
