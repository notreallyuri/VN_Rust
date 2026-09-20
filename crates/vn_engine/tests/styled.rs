use vn_engine::screens::playing::Typewriter;
use vn_engine::ui::styled::StyledText;

#[test]
fn visible_length_counts_text_not_tags() {
    let styled = StyledText::parse("[b]Mary[/b] said [color=#ff0000]no[/color].");
    assert_eq!(styled.plain(), "Mary said no.");
    assert_eq!(styled.chars(), "Mary said no.".chars().count());
}

#[test]
fn pauses_are_listed_by_the_character_they_wait_at() {
    let styled = StyledText::parse("Wait[w=0.75] for it");
    assert_eq!(styled.pauses(), [(4, 0.75)]);

    let leading = StyledText::parse("[w=1]No pause before the start");
    assert_eq!(leading.pauses(), []);
}

#[test]
fn the_typewriter_reveals_visible_characters_only() {
    let text = "[b]Mary[/b] said no.";
    let typewriter = Typewriter::start(text, 10, 0.0);

    assert_eq!(typewriter.visible(0.0), 0);
    assert_eq!(typewriter.visible(0.4), 4);
    assert_eq!(typewriter.visible(1.3), 13);
    assert!(typewriter.is_done(1.3));
    assert_eq!(typewriter.visible(99.0), "Mary said no.".chars().count());
}

#[test]
fn a_wait_tag_holds_the_typewriter_then_carries_on() {
    let typewriter = Typewriter::start("abcd[w=1]efgh", 10, 0.0);

    assert_eq!(typewriter.visible(0.2), 2);
    assert_eq!(typewriter.visible(0.4), 4);
    assert_eq!(typewriter.visible(0.9), 4, "still waiting");
    assert_eq!(typewriter.visible(1.4), 4, "still waiting");
    assert_eq!(typewriter.visible(1.6), 6);
    assert!(typewriter.is_done(2.4));
}

#[test]
fn a_finished_typewriter_shows_everything() {
    let typewriter = Typewriter::finished("[i]all of it[/i]");
    assert_eq!(typewriter.visible(0.0), "all of it".chars().count());
    assert!(typewriter.is_done(0.0));
}
