use novn::dev::inspector::{self, Timing, Widget, colour, under};
use novn::input::navigation::{Focus, NavInput};
use novn::raylib::prelude::{Color, Rectangle, Vector2};

fn button(label: &str, rect: Rectangle) -> Widget {
    Widget {
        kind: "button",
        rect,
        label: label.into(),
        ..Widget::default()
    }
}

#[test]
fn nothing_is_recorded_while_it_is_off() {
    inspector::set_active(false);
    inspector::widget(button("New Game", Rectangle::new(0.0, 0.0, 10.0, 10.0)));
    inspector::focus_list(&[Rectangle::new(0.0, 0.0, 10.0, 10.0)], &[true], None);
    let frame = inspector::frame();
    assert!(frame.widgets.is_empty());
    assert!(frame.focus.is_empty());
}

#[test]
fn a_frame_collects_what_was_drawn_and_the_next_one_starts_clean() {
    inspector::set_active(true);
    inspector::begin_frame();
    inspector::widget(button("New Game", Rectangle::new(0.0, 0.0, 10.0, 10.0)));
    inspector::widget(button("Quit", Rectangle::new(0.0, 20.0, 10.0, 10.0)));
    assert_eq!(inspector::frame().widgets.len(), 2);

    inspector::begin_frame();
    assert!(inspector::frame().widgets.is_empty());
    inspector::set_active(false);
}

#[test]
fn the_widget_under_the_pointer_is_the_one_drawn_last() {
    let widgets = [
        button("panel", Rectangle::new(0.0, 0.0, 100.0, 100.0)),
        button("inside", Rectangle::new(10.0, 10.0, 20.0, 20.0)),
    ];
    assert_eq!(
        under(&widgets, Vector2::new(15.0, 15.0)).unwrap().label,
        "inside"
    );
    assert_eq!(
        under(&widgets, Vector2::new(80.0, 80.0)).unwrap().label,
        "panel"
    );
    assert!(under(&widgets, Vector2::new(500.0, 5.0)).is_none());
}

#[test]
fn focus_is_recorded_in_tab_order_even_with_nothing_focused() {
    inspector::set_active(true);
    inspector::begin_frame();
    let rects = [
        Rectangle::new(0.0, 0.0, 10.0, 10.0),
        Rectangle::new(0.0, 20.0, 10.0, 10.0),
        Rectangle::new(0.0, 40.0, 10.0, 10.0),
    ];
    let mut focus = Focus::default();
    let pointer = NavInput {
        pointer: true,
        ..NavInput::default()
    };
    focus.update(&pointer, &rects, &[true, false, true], None);

    let frame = inspector::frame();
    assert_eq!(
        frame.focus.len(),
        1,
        "a mouse user with nothing focused still sees the order"
    );
    assert_eq!(frame.focus[0].rects, rects);
    assert_eq!(frame.focus[0].enabled, [true, false, true]);
    assert_eq!(frame.focus[0].focused, None);
    inspector::set_active(false);
}

#[test]
fn timing_reports_the_rate_the_average_and_the_worst_frame() {
    let mut timing = Timing::default();
    assert!(timing.summary().is_none());
    for _ in 0..9 {
        timing.push(0.016);
    }
    timing.push(0.050);
    let summary = timing.summary().unwrap();
    assert!(
        (summary.average_ms - 19.4).abs() < 0.01,
        "{}",
        summary.average_ms
    );
    assert!((summary.worst_ms - 50.0).abs() < 0.01);
    assert!((summary.fps - 1000.0 / 19.4).abs() < 0.1);
}

#[test]
fn timing_keeps_a_fixed_window_and_ignores_nonsense() {
    let mut timing = Timing::default();
    for _ in 0..500 {
        timing.push(0.016);
    }
    timing.push(0.0);
    timing.push(f32::NAN);
    timing.push(-1.0);
    assert_eq!(timing.samples().count(), inspector::SAMPLES);
}

#[test]
fn colours_read_like_css() {
    assert_eq!(colour(Color::new(255, 190, 92, 255)), "#ffbe5c");
    assert_eq!(colour(Color::new(10, 10, 20, 220)), "#0a0a14dc");
}

#[test]
fn the_frame_graph_steps_aside_from_the_pointer() {
    use novn::dev::inspector::graph_rect;
    let screen = Vector2::new(1280.0, 720.0);
    let away = graph_rect(screen, Vector2::new(400.0, 500.0));
    assert!(away.x > 1000.0, "top-right by default, got {away:?}");
    let near = graph_rect(screen, Vector2::new(1200.0, 30.0));
    assert!(
        near.x < 100.0,
        "top-left when the pointer is there, got {near:?}"
    );
    assert!(
        !near.check_collision_point_rec(Vector2::new(1200.0, 30.0)),
        "and never under the pointer"
    );
}
