use raylib::prelude::*;
use vn_engine::frame::viewport::Viewport;
use vn_engine::ui::scroll::{Scroll, ScrollStyle};

fn area() -> Rectangle {
    Rectangle::new(100.0, 50.0, 400.0, 200.0)
}

#[test]
fn content_that_fits_does_not_scroll() {
    let mut scroll = Scroll::new();
    scroll.extent(200.0, 150.0);
    assert_eq!(scroll.max(), 0.0);
    assert!(!scroll.overflows());
    scroll.by(500.0);
    assert_eq!(scroll.offset(), 0.0);
    assert!(scroll.bar(area(), &ScrollStyle::default()).is_none());
}

#[test]
fn scrolling_is_clamped_to_the_content() {
    let mut scroll = Scroll::new();
    scroll.extent(200.0, 500.0);
    assert_eq!(scroll.max(), 300.0);

    scroll.by(100.0);
    assert_eq!(scroll.offset(), 100.0);
    assert_eq!(scroll.from_end(), 200.0);

    scroll.by(1000.0);
    assert_eq!(scroll.offset(), 300.0);
    assert!(scroll.at_end());

    scroll.by(-1000.0);
    assert_eq!(scroll.offset(), 0.0);
}

#[test]
fn shrinking_content_pulls_the_offset_back() {
    let mut scroll = Scroll::new();
    scroll.extent(200.0, 500.0);
    scroll.to_end();
    assert_eq!(scroll.offset(), 300.0);

    scroll.extent(200.0, 250.0);
    assert_eq!(scroll.offset(), 50.0);

    scroll.extent(200.0, 100.0);
    assert_eq!(scroll.offset(), 0.0);
}

#[test]
fn the_thumb_shrinks_as_the_content_grows_and_never_leaves_the_bar() {
    let style = ScrollStyle::default();
    let mut scroll = Scroll::new();

    scroll.extent(200.0, 400.0);
    let half = scroll.thumb(area(), &style).unwrap();
    assert_eq!(half.height, 100.0);
    assert_eq!(half.y, area().y);

    scroll.extent(200.0, 2000.0);
    let small = scroll.thumb(area(), &style).unwrap();
    assert!(small.height >= style.min_thumb);
    assert!(small.height < half.height);

    scroll.to_end();
    let bottom = scroll.thumb(area(), &style).unwrap();
    assert_eq!(bottom.y + bottom.height, area().y + area().height);
}

#[test]
fn a_page_is_most_of_the_view() {
    let mut scroll = Scroll::new();
    scroll.extent(300.0, 900.0);
    assert_eq!(scroll.page(), 270.0);
}

#[test]
fn a_viewport_maps_window_points_back_into_the_game() {
    let viewport = Viewport {
        size: (1280, 720),
        destination: Rectangle::new(320.0, 0.0, 1280.0, 720.0),
    };
    assert_eq!(viewport.scale(), 1.0);
    let point = viewport.to_viewport(Vector2::new(320.0, 10.0));
    assert_eq!((point.x, point.y), (0.0, 10.0));
}

#[test]
fn a_scaled_viewport_maps_points_back_to_design_coordinates() {
    let viewport = Viewport {
        size: (1280, 720),
        destination: Rectangle::new(0.0, 180.0, 2560.0, 1440.0),
    };
    assert_eq!(viewport.scale(), 2.0);

    let centre = viewport.to_viewport(Vector2::new(1280.0, 900.0));
    assert_eq!((centre.x, centre.y), (640.0, 360.0));

    let corner = viewport.to_viewport(Vector2::new(0.0, 180.0));
    assert_eq!((corner.x, corner.y), (0.0, 0.0));
}
