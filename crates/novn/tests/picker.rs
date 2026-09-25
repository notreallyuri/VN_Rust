use novn::dev::picker::{
    self, PositionPickerOverlay, dragged_x, hit, nearest, rust_line, story_line,
};
use novn::raylib::prelude::{Rectangle, Vector2};
use novn::script::Position;

const SLOTS: [f32; 5] = [0.1, 0.25, 0.5, 0.75, 0.9];

#[test]
fn a_drop_snaps_to_the_nearest_slot() {
    assert_eq!(nearest(&SLOTS, 0.0), Position::FarLeft);
    assert_eq!(nearest(&SLOTS, 0.2), Position::Left);
    assert_eq!(nearest(&SLOTS, 0.52), Position::Center);
    assert_eq!(nearest(&SLOTS, 0.8), Position::Right);
    assert_eq!(nearest(&SLOTS, 1.0), Position::FarRight);
}

#[test]
fn the_lines_are_ones_you_can_paste() {
    assert_eq!(
        story_line("mary", "tired", Position::FarLeft),
        "show mary tired at far_left"
    );
    assert_eq!(
        rust_line(Position::Left, 0.3125),
        ".position(Position::Left, 0.312)"
    );
}

#[test]
fn the_character_in_front_is_the_one_picked_up() {
    let drawn = [
        picker::Drawn {
            character: "mary".into(),
            image: "tired".into(),
            rect: Rectangle::new(0.0, 0.0, 200.0, 400.0),
        },
        picker::Drawn {
            character: "hugo".into(),
            image: "neutral".into(),
            rect: Rectangle::new(150.0, 0.0, 200.0, 400.0),
        },
    ];
    assert_eq!(
        hit(&drawn, Vector2::new(175.0, 100.0)).unwrap().character,
        "hugo"
    );
    assert_eq!(
        hit(&drawn, Vector2::new(50.0, 100.0)).unwrap().character,
        "mary"
    );
    assert!(hit(&drawn, Vector2::new(500.0, 100.0)).is_none());
}

#[test]
fn nothing_is_recorded_until_the_picker_listens() {
    picker::listen(false);
    picker::record("mary", "tired", Rectangle::new(0.0, 0.0, 10.0, 10.0));
    picker::record_slots(SLOTS);
    assert!(picker::current().characters.is_empty());
    assert!(picker::current().slots.is_none());
}

#[test]
fn the_last_frame_is_kept_for_the_update_that_follows() {
    picker::listen(true);
    picker::begin_frame();
    picker::record("mary", "tired", Rectangle::new(0.0, 0.0, 10.0, 10.0));
    picker::record_slots(SLOTS);
    picker::begin_frame();
    assert_eq!(picker::last().characters.len(), 1, "what was drawn is kept");
    assert_eq!(picker::last().slots, Some(SLOTS));
    assert!(
        picker::current().characters.is_empty(),
        "and the new frame starts empty"
    );
    picker::listen(false);
}

#[test]
fn a_drag_moves_only_the_character_being_dragged() {
    picker::listen(true);
    picker::set_drag(Some(("mary", 0.4)));
    assert_eq!(dragged_x("mary"), Some(0.4));
    assert_eq!(dragged_x("hugo"), None);
    picker::listen(false);
}

#[test]
fn closing_the_picker_always_puts_the_character_back() {
    picker::listen(true);
    let overlay = PositionPickerOverlay::new();
    picker::set_drag(Some(("mary", 0.4)));
    drop(overlay);
    assert_eq!(
        dragged_x("mary"),
        None,
        "a screen change drops the overlay without closing it"
    );
    assert!(!picker::listening());
}
