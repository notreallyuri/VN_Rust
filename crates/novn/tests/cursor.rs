use novn::raylib::prelude::Vector2;
use novn::request::{Request, Requests};
use novn::screen::ScreenState;
use novn::ui::cursor::{CursorKind, CursorStyle, effective, system_shape};

fn asked(kinds: &[CursorKind]) -> CursorKind {
    let mut list = Requests::default();
    for &kind in kinds {
        list.push(Request::Cursor(kind));
    }
    list.resolve().cursor
}

#[test]
fn the_strongest_request_of_a_frame_wins_whatever_the_order() {
    use CursorKind::*;
    assert_eq!(asked(&[Hand, Text]), Hand);
    assert_eq!(asked(&[Text, Hand]), Hand);
    assert_eq!(asked(&[Grab, Hand, Arrow]), Grab);
    assert_eq!(
        asked(&[Grabbing, NotAllowed]),
        NotAllowed,
        "dragging over a target that refuses the item says so"
    );
    assert_eq!(asked(&[Custom("examine"), Hand]), Custom("examine"));
    assert_eq!(asked(&[Grabbing, Custom("examine")]), Grabbing);
    assert_eq!(asked(&[NotAllowed, Hidden]), Hidden);
    assert_eq!(asked(&[]), Arrow);
}

#[test]
fn a_state_without_a_picture_falls_back_to_the_nearest_one() {
    let only_arrow = CursorStyle::new("arrow.png");
    for kind in [
        CursorKind::Text,
        CursorKind::Hand,
        CursorKind::Grab,
        CursorKind::Grabbing,
        CursorKind::NotAllowed,
        CursorKind::Custom("examine"),
    ] {
        assert_eq!(only_arrow.file(kind), Some("arrow.png"), "{kind:?}");
    }

    let hands = CursorStyle::new("arrow.png")
        .hand("hand.png")
        .picture(CursorKind::Grab, "grab.png");
    assert_eq!(
        hands.file(CursorKind::Grabbing),
        Some("grab.png"),
        "grabbing borrows the open hand"
    );
    assert_eq!(hands.file(CursorKind::Custom("examine")), Some("hand.png"));
    assert_eq!(hands.file(CursorKind::NotAllowed), Some("arrow.png"));
    assert_eq!(
        hands.file(CursorKind::Hidden),
        None,
        "hidden has no picture"
    );
}

#[test]
fn a_custom_cursor_is_found_by_its_name() {
    let style = CursorStyle::new("arrow.png")
        .picture(CursorKind::Custom("examine"), "magnifier.png")
        .picture(CursorKind::Custom("talk"), "speech.png");
    assert_eq!(
        style.file(CursorKind::Custom("examine")),
        Some("magnifier.png")
    );
    assert_eq!(style.file(CursorKind::Custom("talk")), Some("speech.png"));
    assert_eq!(style.file(CursorKind::Custom("unknown")), Some("arrow.png"));
}

#[test]
fn each_picture_can_click_from_its_own_point() {
    let style = CursorStyle::new("arrow.png")
        .picture(CursorKind::Text, "beam.png")
        .size(32.0)
        .hotspot(0.1, 0.1)
        .hotspot_for(CursorKind::Text, 0.5, 0.5);
    let square = Vector2::new(32.0, 32.0);

    let arrow = style.pixels(CursorKind::Arrow, square, 1.0);
    let beam = style.pixels(CursorKind::Text, square, 1.0);
    assert_eq!(
        (arrow.hot_x, arrow.hot_y),
        (3, 3),
        "the arrow clicks at its tip"
    );
    assert_eq!(
        (beam.hot_x, beam.hot_y),
        (16, 16),
        "the I-beam clicks at its centre"
    );

    let borrowed = style.pixels(CursorKind::Hand, square, 1.0);
    assert_eq!(
        (borrowed.hot_x, borrowed.hot_y),
        (3, 3),
        "a state drawn with another's picture uses that picture's hotspot"
    );
}

#[test]
fn a_forced_cursor_belongs_to_the_screen_that_forced_it() {
    let playing = ScreenState::Playing;
    let forced = (playing.clone(), CursorKind::Custom("examine"));

    assert_eq!(
        effective(Some(&forced), &playing, false, CursorKind::Hand),
        CursorKind::Custom("examine"),
        "it beats whatever widgets asked for"
    );
    assert_eq!(
        effective(Some(&forced), &playing, true, CursorKind::Hand),
        CursorKind::Hand,
        "but not over the pause menu or another overlay"
    );
    assert_eq!(
        effective(
            Some(&forced),
            &ScreenState::MainMenu,
            false,
            CursorKind::Arrow
        ),
        CursorKind::Arrow,
        "and not on another screen"
    );
    assert_eq!(
        effective(None, &playing, false, CursorKind::Grab),
        CursorKind::Grab
    );
}

#[test]
fn forcing_and_releasing_is_asked_for_like_everything_else() {
    let mut list = Requests::default();
    list.push(Request::OverrideCursor(Some(CursorKind::Hidden)));
    assert_eq!(
        list.resolve().cursor_override,
        Some(Some(CursorKind::Hidden))
    );

    let mut list = Requests::default();
    list.push(Request::OverrideCursor(Some(CursorKind::Hand)));
    list.push(Request::OverrideCursor(None));
    assert_eq!(
        list.resolve().cursor_override,
        Some(None),
        "the last word wins"
    );

    assert_eq!(
        Requests::default().resolve().cursor_override,
        None,
        "a quiet frame changes nothing"
    );
}

#[test]
fn a_game_without_pictures_gets_the_systems_own_shapes() {
    use novn::raylib::consts::MouseCursor::*;
    assert_eq!(system_shape(CursorKind::Arrow), Some(MOUSE_CURSOR_DEFAULT));
    assert_eq!(system_shape(CursorKind::Text), Some(MOUSE_CURSOR_IBEAM));
    assert_eq!(
        system_shape(CursorKind::Hand),
        Some(MOUSE_CURSOR_POINTING_HAND)
    );
    assert_eq!(
        system_shape(CursorKind::Grab),
        Some(MOUSE_CURSOR_POINTING_HAND)
    );
    assert_eq!(
        system_shape(CursorKind::Grabbing),
        Some(MOUSE_CURSOR_RESIZE_ALL)
    );
    assert_eq!(
        system_shape(CursorKind::NotAllowed),
        Some(MOUSE_CURSOR_NOT_ALLOWED)
    );
    assert_eq!(
        system_shape(CursorKind::Custom("x")),
        Some(MOUSE_CURSOR_POINTING_HAND)
    );
    assert_eq!(system_shape(CursorKind::Hidden), None);
}
