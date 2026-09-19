use vn_engine::raylib::prelude::{Rectangle, Vector2};
use vn_engine::{Focus, NavInput, Repeater, navigate};

fn column(count: usize) -> Vec<Rectangle> {
    (0..count)
        .map(|i| Rectangle::new(500.0, 100.0 + i as f32 * 60.0, 240.0, 50.0))
        .collect()
}

fn grid(columns: usize, count: usize) -> Vec<Rectangle> {
    (0..count)
        .map(|i| {
            let (col, row) = (i % columns, i / columns);
            Rectangle::new(
                100.0 + col as f32 * 300.0,
                100.0 + row as f32 * 100.0,
                280.0,
                80.0,
            )
        })
        .collect()
}

const UP: Vector2 = Vector2 { x: 0.0, y: -1.0 };
const DOWN: Vector2 = Vector2 { x: 0.0, y: 1.0 };
const LEFT: Vector2 = Vector2 { x: -1.0, y: 0.0 };
const RIGHT: Vector2 = Vector2 { x: 1.0, y: 0.0 };

#[test]
fn columns_move_up_and_down_and_wrap() {
    let rects = column(4);
    assert_eq!(navigate(&rects, &[], 0, DOWN, true), Some(1));
    assert_eq!(navigate(&rects, &[], 2, UP, true), Some(1));
    assert_eq!(
        navigate(&rects, &[], 3, DOWN, true),
        Some(0),
        "wraps to the top"
    );
    assert_eq!(
        navigate(&rects, &[], 0, UP, true),
        Some(3),
        "and to the bottom"
    );
    assert_eq!(navigate(&rects, &[], 3, DOWN, false), None);
    assert_eq!(
        navigate(&rects, &[], 1, LEFT, true),
        None,
        "nothing beside it"
    );
}

#[test]
fn grids_move_in_both_directions() {
    let rects = grid(2, 6);
    assert_eq!(navigate(&rects, &[], 0, RIGHT, true), Some(1));
    assert_eq!(
        navigate(&rects, &[], 1, DOWN, true),
        Some(3),
        "straight down, not diagonal"
    );
    assert_eq!(navigate(&rects, &[], 3, LEFT, true), Some(2));
    assert_eq!(
        navigate(&rects, &[], 4, DOWN, true),
        Some(0),
        "wraps within the column"
    );
}

#[test]
fn a_wide_button_above_a_row_of_two_goes_to_the_row() {
    let wide = |y: f32| Rectangle::new(368.0, y, 544.0, 54.0);
    let left = |y: f32| Rectangle::new(368.0, y, 264.0, 54.0);
    let right = |y: f32| Rectangle::new(648.0, y, 264.0, 54.0);
    let menu = [
        wide(322.0),
        left(390.0),
        right(390.0),
        left(458.0),
        right(458.0),
        wide(526.0),
    ];
    let (new_game, continue_, load, settings, credits, exit) = (0, 1, 2, 3, 4, 5);

    assert_eq!(navigate(&menu, &[], new_game, DOWN, true), Some(continue_));
    assert_eq!(navigate(&menu, &[], new_game, LEFT, true), None);
    assert_eq!(navigate(&menu, &[], new_game, RIGHT, true), None);
    assert_eq!(
        navigate(&menu, &[], new_game, UP, true),
        Some(exit),
        "wraps"
    );
    assert_eq!(navigate(&menu, &[], exit, UP, true), Some(settings));
    assert_eq!(navigate(&menu, &[], continue_, RIGHT, true), Some(load));
    assert_eq!(
        navigate(&menu, &[], load, RIGHT, true),
        Some(continue_),
        "wraps"
    );
    assert_eq!(navigate(&menu, &[], load, DOWN, true), Some(credits));
    assert_eq!(navigate(&menu, &[], credits, UP, true), Some(load));
    assert_eq!(navigate(&menu, &[], load, UP, true), Some(new_game));
    assert_eq!(navigate(&menu, &[], credits, DOWN, true), Some(exit));
    assert_eq!(
        navigate(
            &menu,
            &[true, false, true, true, true, true],
            new_game,
            DOWN,
            true
        ),
        Some(load),
        "a disabled Continue is skipped"
    );
}

#[test]
fn disabled_items_are_skipped() {
    let rects = column(4);
    let enabled = [true, false, true, true];
    assert_eq!(navigate(&rects, &enabled, 0, DOWN, true), Some(2));
    assert_eq!(navigate(&rects, &enabled, 2, UP, true), Some(0));
}

fn keys(apply: impl FnOnce(&mut NavInput)) -> NavInput {
    let mut nav = NavInput::default();
    apply(&mut nav);
    nav
}

#[test]
fn focus_starts_on_the_first_usable_item_and_follows_keys() {
    let rects = column(3);
    let enabled = [false, true, true];
    let mut focus = Focus::default();

    let pointer = keys(|n| n.pointer = true);
    assert_eq!(focus.update(&pointer, &rects, &enabled, None), None);
    assert_eq!(focus.index(), None, "the mouse alone doesn't focus");

    let down = keys(|n| n.down = true);
    assert_eq!(focus.update(&down, &rects, &enabled, None), None);
    assert_eq!(
        focus.index(),
        Some(1),
        "the first press only shows the focus"
    );

    focus.update(&down, &rects, &enabled, None);
    assert_eq!(focus.index(), Some(2));

    let accept = keys(|n| n.accept = true);
    assert_eq!(focus.update(&accept, &rects, &enabled, None), Some(2));

    let tab = keys(|n| n.next = true);
    focus.update(&tab, &rects, &enabled, None);
    assert_eq!(
        focus.index(),
        Some(1),
        "Tab cycles and skips disabled items"
    );
}

#[test]
fn focus_follows_the_mouse_and_shows_itself_in_keyboard_mode() {
    let rects = column(3);
    let mut focus = Focus::default();

    let hover = keys(|n| n.pointer = true);
    focus.update(&hover, &rects, &[], Some(2));
    assert_eq!(focus.index(), Some(2));

    let keyboard = NavInput::default();
    let mut fresh = Focus::default();
    fresh.update(&keyboard, &rects, &[], None);
    assert_eq!(
        fresh.index(),
        Some(0),
        "a new menu in keyboard mode focuses its first item"
    );

    focus.update(&keyboard, &column(2), &[], None);
    assert_eq!(
        focus.index(),
        Some(0),
        "a focus past the end is dropped and restarted"
    );
}

#[test]
fn held_directions_repeat_after_a_delay() {
    let mut repeater = Repeater::default();
    let fire = |r: &mut Repeater, held, now| r.update(held, now, 0.4, 0.1);

    assert!(fire(&mut repeater, true, 0.0));
    assert!(!fire(&mut repeater, true, 0.2));
    assert!(fire(&mut repeater, true, 0.41));
    assert!(!fire(&mut repeater, true, 0.45));
    assert!(fire(&mut repeater, true, 0.52));
    assert!(!fire(&mut repeater, false, 0.6));
    assert!(fire(&mut repeater, true, 0.61), "a new press fires at once");
}

#[test]
fn nav_input_helpers() {
    assert_eq!(keys(|n| n.up = true).direction(), Some(UP));
    assert_eq!(keys(|n| n.right = true).horizontal(), 1);
    assert_eq!(keys(|n| n.up = true).vertical(), -1);
    assert_eq!(keys(|n| n.pointer = true).direction(), None);
    assert!(!keys(|n| n.pointer = true).any_key());
    assert!(keys(|n| n.page_back = true).any_key());
}
