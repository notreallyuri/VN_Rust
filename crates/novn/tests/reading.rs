use novn::raylib::prelude::{Color, Vector2};
use novn::screens::playing::PlayingConfig;
use novn::ui::TextStyle;
use novn::ui::fonts::FontRole;
use novn::ui::reading;

#[test]
fn reading_text_scales_with_the_setting() {
    let style = TextStyle::new(FontRole::Dialogue, 26.0, Color::WHITE);
    reading::set_percent(100);
    assert_eq!(reading::text(&style).size, 26.0);
    reading::set_percent(150);
    assert_eq!(reading::text(&style).size, 39.0);
    reading::set_percent(999);
    assert_eq!(reading::scale(), 1.5, "clamped to the largest size offered");
    reading::set_percent(100);
}

#[test]
fn the_dialogue_box_grows_upward_to_fit_bigger_text() {
    let config = PlayingConfig::default();
    let screen = Vector2::new(1280.0, 720.0);
    reading::set_percent(100);
    let normal = config.dialogue_box.rect(screen);
    reading::set_percent(150);
    let large = config.dialogue_box.rect(screen);
    reading::set_percent(100);

    assert!(large.height > normal.height);
    assert_eq!(
        large.y + large.height,
        normal.y + normal.height,
        "the bottom edge stays put, so it grows upward"
    );
    let inside = |r: novn::raylib::prelude::Rectangle| r.height - config.dialogue_box.padding * 2.0;
    assert!(
        (inside(large) - inside(normal) * 1.5).abs() < 0.01,
        "the text area grows with the text, the padding does not"
    );
}

#[test]
fn choices_grow_with_the_text_but_never_past_the_screen() {
    let config = PlayingConfig::default();
    let screen = Vector2::new(1280.0, 720.0);
    reading::set_percent(100);
    let normal = config.choice_rects(3, screen);
    reading::set_percent(150);
    let large = config.choice_rects(3, screen);
    reading::set_percent(100);
    assert!(large[0].height > normal[0].height);
    assert!(
        large
            .iter()
            .all(|r| r.x >= 0.0 && r.x + r.width <= screen.x)
    );
}
