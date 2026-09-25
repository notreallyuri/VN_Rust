use novn::dev::inspector::Styled;
use novn::dev::styler::{self, Channel, Edit, Knob, StyleEditorOverlay, knobs, roundness_of};
use novn::raylib::prelude::Color;
use novn::ui::button::ButtonStyle;
use novn::ui::shape::{CornerShape, Corners, PanelStyle};

fn button() -> Styled {
    Styled::Button(Box::new(
        ButtonStyle::default().size(124.0, 36.0).font_size(15.0),
    ))
}

fn as_button(styled: &Styled) -> &ButtonStyle {
    match styled {
        Styled::Button(b) => b,
        Styled::Panel(_) => panic!("not a button"),
    }
}

#[test]
fn the_preview_is_exactly_what_the_copied_calls_build() {
    let original = button();
    let mut edit = Edit::default();
    edit.nudge(&original, Knob::Width, 16.0);
    edit.nudge(&original, Knob::Fill(Channel::R), 100.0);
    edit.nudge(&original, Knob::Roundness, 0.3);

    let fill = Color::new(140, 40, 60, 255);
    assert_eq!(
        edit.rust(),
        ".size(140.0, 36.0).color(Color::new(140, 40, 60, 255)).roundness(0.30)"
    );
    let by_hand = as_button(&original)
        .clone()
        .size(140.0, 36.0)
        .color(fill)
        .roundness(0.30);
    assert_eq!(
        as_button(&edit.apply(&original)),
        &by_hand,
        "the preview goes through the same builder calls it copies"
    );
}

#[test]
fn a_fill_change_moves_the_hover_fill_with_it_as_the_builder_does() {
    let original = button();
    let mut edit = Edit::default();
    edit.nudge(&original, Knob::Fill(Channel::G), 50.0);
    let edited = edit.apply(&original);
    let before = as_button(&original).hovered.fill;
    let after = as_button(&edited).hovered.fill;
    assert_ne!(
        before, after,
        "setting the field alone would leave hover stale"
    );
}

#[test]
fn values_stay_in_range() {
    let original = button();
    let mut edit = Edit::default();
    edit.nudge(&original, Knob::Fill(Channel::A), 9999.0);
    assert_eq!(
        Edit::value(&edit.apply(&original), Knob::Fill(Channel::A)),
        Some(255.0)
    );
    edit.nudge(&original, Knob::Roundness, -5.0);
    assert_eq!(
        Edit::value(&edit.apply(&original), Knob::Roundness),
        Some(0.0)
    );
    edit.nudge(&original, Knob::Width, -99999.0);
    assert_eq!(Edit::value(&edit.apply(&original), Knob::Width), Some(0.0));
}

#[test]
fn a_border_can_be_added_where_there_was_none() {
    let original = button();
    assert_eq!(Edit::value(&original, Knob::BorderWidth), Some(0.0));
    let mut edit = Edit::default();
    edit.nudge(&original, Knob::BorderWidth, 2.0);
    assert_eq!(edit.rust(), ".border(2.0, Color::new(255, 255, 255, 255))");
}

#[test]
fn only_uniform_round_corners_read_as_a_roundness() {
    assert_eq!(roundness_of(&Corners::SQUARE), Some(0.0));
    assert_eq!(roundness_of(&Corners::roundness(0.4)), Some(0.4));
    let mixed = Corners::roundness(0.4).top_left(CornerShape::Bevel, 8.0);
    assert_eq!(
        roundness_of(&mixed),
        None,
        "shown as custom rather than guessed"
    );
}

#[test]
fn a_panel_offers_only_what_a_panel_has() {
    let panel = Styled::Panel(PanelStyle::new(Color::new(20, 16, 12, 240)));
    let offered = knobs(&panel);
    assert!(!offered.contains(&Knob::Width));
    assert!(!offered.contains(&Knob::FontSize));
    assert!(offered.contains(&Knob::Fill(Channel::A)));
    assert!(knobs(&button()).contains(&Knob::PaddingX));
}

#[test]
fn nothing_is_overridden_outside_the_editor() {
    styler::clear();
    let style = ButtonStyle::default();
    assert!(styler::button(&style).is_none());
}

#[test]
fn closing_the_editor_puts_every_style_back() {
    let overlay = StyleEditorOverlay::new();
    drop(overlay);
    assert!(!styler::editing());
    assert!(styler::button(&ButtonStyle::default()).is_none());
}

#[test]
fn a_size_change_previews_around_the_buttons_centre() {
    use novn::dev::styler::resized;
    use novn::raylib::prelude::Rectangle;
    let original = ButtonStyle::default().size(100.0, 40.0);
    let wider = original.clone().size(150.0, 40.0);
    let rect = resized(Rectangle::new(200.0, 100.0, 100.0, 40.0), &original, &wider);
    assert_eq!(
        (rect.x, rect.width),
        (175.0, 150.0),
        "grows equally both ways"
    );
    let squeezed = resized(Rectangle::new(0.0, 0.0, 80.0, 40.0), &original, &wider);
    assert_eq!(
        squeezed.width, 120.0,
        "a layout squeezing it keeps squeezing it"
    );
}

#[test]
fn the_panel_keeps_clear_of_every_widget_being_edited() {
    use novn::dev::styler::{placement, union};
    use novn::raylib::prelude::{Rectangle, Vector2};
    let screen = Vector2::new(1280.0, 720.0);
    let overlaps = |a: Rectangle, b: Rectangle| a.check_collision_recs(&b);

    let some = union([
        Rectangle::new(166.0, 660.0, 124.0, 36.0),
        Rectangle::new(590.0, 660.0, 124.0, 36.0),
    ])
    .unwrap();
    assert_eq!(some, Rectangle::new(166.0, 660.0, 548.0, 36.0));
    let panel = placement(screen, Some(some), 330.0, 16.0);
    assert!(!overlaps(panel, some), "{panel:?} covers {some:?}");

    let whole_bar = Rectangle::new(40.0, 660.0, 1200.0, 36.0);
    let panel = placement(screen, Some(whole_bar), 330.0, 16.0);
    assert!(
        !overlaps(panel, whole_bar),
        "a full-width bar shortens the panel: {panel:?}"
    );

    let top_bar = Rectangle::new(40.0, 10.0, 1200.0, 40.0);
    let panel = placement(screen, Some(top_bar), 330.0, 16.0);
    assert!(
        !overlaps(panel, top_bar),
        "and a bar along the top pushes it down: {panel:?}"
    );

    let aside = placement(
        screen,
        Some(Rectangle::new(80.0, 200.0, 200.0, 300.0)),
        330.0,
        16.0,
    );
    assert!(
        aside.x > 280.0,
        "a widget on the left puts the panel on the right"
    );
}
