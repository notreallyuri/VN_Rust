use vn_script::markup::{parse, plain, validate};

#[test]
fn text_without_tags_is_one_span() {
    let spans = parse("Just a line.");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].text, "Just a line.");
    assert!(!spans[0].bold);
    assert_eq!(spans[0].color, None);
}

#[test]
fn bold_and_italic_mark_the_text_between_them() {
    let spans = parse("a [b]bold[/b] and [i]slanted[/i] word");
    let styled: Vec<_> = spans
        .iter()
        .map(|s| (s.text.as_str(), s.bold, s.italic))
        .collect();
    assert_eq!(
        styled,
        [
            ("a ", false, false),
            ("bold", true, false),
            (" and ", false, false),
            ("slanted", false, true),
            (" word", false, false),
        ]
    );
}

#[test]
fn colour_and_size_nest_and_pop_back() {
    let spans = parse("[color=#c8a165]gold [size=28]big[/size] gold[/color] plain");
    let styled: Vec<_> = spans
        .iter()
        .map(|s| (s.text.as_str(), s.color, s.size))
        .collect();
    assert_eq!(
        styled,
        [
            ("gold ", Some(0xc8a165), None),
            ("big", Some(0xc8a165), Some(28.0)),
            (" gold", Some(0xc8a165), None),
            (" plain", None, None),
        ]
    );
}

#[test]
fn a_wait_tag_pauses_the_span_that_follows() {
    let spans = parse("Wait[w=0.75] for it[w] now");
    assert_eq!(spans[0].text, "Wait");
    assert_eq!(spans[0].pause, 0.0);
    assert_eq!(spans[1].text, " for it");
    assert_eq!(spans[1].pause, 0.75);
    assert_eq!(spans[2].text, " now");
    assert_eq!(spans[2].pause, 0.5);
}

#[test]
fn a_doubled_bracket_is_a_literal_bracket() {
    let spans = parse("a [[note] here");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].text, "a [note] here");
    assert_eq!(plain("a [[note] here"), "a [note] here");
}

#[test]
fn plain_text_drops_every_tag() {
    assert_eq!(
        plain("[b]Mary[/b] said [color=#ff0000]no[/color][w=1]."),
        "Mary said no."
    );
}

#[test]
fn interpolation_braces_are_left_alone() {
    let spans = parse("Hello {player_name}, [b]welcome[/b]");
    assert_eq!(spans[0].text, "Hello {player_name}, ");
    assert_eq!(spans[1].text, "welcome");
}

#[test]
fn a_good_line_has_nothing_to_report() {
    assert_eq!(
        validate("[b]Fine[/b] and [color=#102030]fine[/color]", 1),
        []
    );
    assert_eq!(validate("No tags at all", 1), []);
    assert_eq!(validate("A [[literal] bracket", 1), []);
}

#[test]
fn unknown_tags_are_reported_with_a_suggestion() {
    let found = validate("[bold]no such tag[/bold]", 7);
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].line, 7);
    assert!(
        found[0].message.contains("unknown text tag `[bold]`"),
        "{}",
        found[0].message
    );

    let near = validate("[colour=#ffffff]x[/color]", 7);
    assert!(
        near[0].message.contains("did you mean 'color'"),
        "{}",
        near[0].message
    );
}

#[test]
fn a_bad_colour_or_size_is_reported() {
    let colour = validate("[color=red]x[/color]", 2);
    assert_eq!(colour.len(), 1);
    assert!(
        colour[0].message.contains("isn't a colour"),
        "{}",
        colour[0].message
    );

    let size = validate("[size=big]x[/size]", 2);
    assert!(
        size[0].message.contains("isn't a text size"),
        "{}",
        size[0].message
    );

    let missing = validate("[color]x[/color]", 2);
    assert!(
        missing[0].message.contains("needs a value"),
        "{}",
        missing[0].message
    );
}

#[test]
fn unbalanced_tags_are_reported() {
    let never_closed = validate("[b]forever", 3);
    assert_eq!(never_closed.len(), 1);
    assert!(never_closed[0].message.contains("never closed"));
    assert!(!never_closed[0].is_error());

    let stray = validate("plain[/b]", 3);
    assert!(stray[0].message.contains("doesn't close anything"));
    assert!(stray[0].is_error());

    let unclosed_bracket = validate("half a [b tag", 3);
    assert!(unclosed_bracket[0].message.contains("missing its closing"));
}
