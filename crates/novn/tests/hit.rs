use novn::input::hit;
use novn::input::hit::Shape;
use novn::input::hit::{Highlight, LabelAt, LabelStyle};
use raylib::prelude::*;

fn area() -> Rectangle {
    Rectangle::new(100.0, 50.0, 400.0, 200.0)
}

fn unit() -> Rectangle {
    Rectangle::new(0.0, 0.0, 1.0, 1.0)
}

fn l_shape() -> Shape {
    Shape::polygon([
        (0.0, 0.0),
        (2.0, 0.0),
        (2.0, 1.0),
        (1.0, 1.0),
        (1.0, 2.0),
        (0.0, 2.0),
    ])
}

fn triangle_area(triangle: [Vector2; 3]) -> f32 {
    hit::signed_area(&triangle).abs()
}

fn in_triangle(triangle: [Vector2; 3], point: Vector2) -> bool {
    let side =
        |a: Vector2, b: Vector2| (b.x - a.x) * (point.y - a.y) - (b.y - a.y) * (point.x - a.x);
    let (a, b, c) = (
        side(triangle[0], triangle[1]),
        side(triangle[1], triangle[2]),
        side(triangle[2], triangle[0]),
    );
    (a > 0.0 && b > 0.0 && c > 0.0) || (a < 0.0 && b < 0.0 && c < 0.0)
}

#[test]
fn a_shape_is_placed_in_fractions_of_the_area() {
    let shape = Shape::rect(0.5, 0.25, 0.25, 0.5);
    let bounds = shape.bounds(area());

    assert_eq!(
        (bounds.x, bounds.y, bounds.width, bounds.height),
        (300.0, 100.0, 100.0, 100.0)
    );
    assert!(shape.contains(area(), Vector2::new(350.0, 150.0)));
    assert!(!shape.contains(area(), Vector2::new(299.0, 150.0)));
    assert_eq!(shape.center(area()), Vector2::new(350.0, 150.0));
}

#[test]
fn a_shape_moves_with_the_area_it_is_placed_in() {
    let shape = Shape::rect(0.5, 0.25, 0.25, 0.5);
    let moved = Rectangle::new(
        area().x + 40.0,
        area().y - 10.0,
        area().width,
        area().height,
    );

    assert!(shape.contains(area(), Vector2::new(310.0, 150.0)));
    assert!(!shape.contains(moved, Vector2::new(310.0, 150.0)));
    assert!(shape.contains(moved, Vector2::new(390.0, 140.0)));
    assert_eq!(shape.center(moved), Vector2::new(390.0, 140.0));
}

#[test]
fn a_circle_takes_its_radius_from_the_width() {
    let shape = Shape::circle(0.5, 0.5, 0.1);
    let center = Vector2::new(300.0, 150.0);

    assert_eq!(shape.center(area()), center);
    assert!(shape.contains(area(), Vector2::new(center.x + 39.0, center.y)));
    assert!(!shape.contains(area(), Vector2::new(center.x + 41.0, center.y)));
    assert!(shape.contains(area(), Vector2::new(center.x, center.y + 39.0)));

    let bounds = shape.bounds(area());
    assert_eq!((bounds.width, bounds.height), (80.0, 80.0));
}

#[test]
fn a_polygon_hit_test_follows_its_outline() {
    let shape = l_shape();

    assert!(shape.contains(unit(), Vector2::new(0.5, 0.5)));
    assert!(shape.contains(unit(), Vector2::new(1.5, 0.5)));
    assert!(shape.contains(unit(), Vector2::new(0.5, 1.5)));
    assert!(
        !shape.contains(unit(), Vector2::new(1.5, 1.5)),
        "the notch of an L is outside it"
    );
    assert!(!shape.contains(unit(), Vector2::new(2.5, 0.5)));
}

#[test]
fn image_coordinates_become_fractions() {
    let shape = Shape::rect(320.0, 180.0, 640.0, 360.0).in_image(1280.0, 720.0);
    assert_eq!(shape, Shape::rect(0.25, 0.25, 0.5, 0.5));

    let circle = Shape::circle(640.0, 360.0, 128.0).in_image(1280.0, 720.0);
    assert_eq!(circle, Shape::circle(0.5, 0.5, 0.1));

    let polygon = Shape::polygon([(0.0, 0.0), (640.0, 720.0)]).in_image(1280.0, 720.0);
    assert_eq!(polygon, Shape::polygon([(0.0, 0.0), (0.5, 1.0)]));
}

#[test]
fn picking_takes_the_shape_drawn_last() {
    let shapes = [
        Shape::rect(0.0, 0.0, 1.0, 1.0),
        Shape::rect(0.4, 0.4, 0.2, 0.2),
        Shape::rect(0.0, 0.0, 0.1, 0.1),
    ];

    let pick = |x, y| hit::pick(shapes.iter(), unit(), Vector2::new(x, y));
    assert_eq!(pick(0.5, 0.5), Some(1));
    assert_eq!(pick(0.05, 0.05), Some(2));
    assert_eq!(pick(0.8, 0.8), Some(0));
    assert_eq!(pick(1.5, 0.5), None);
}

#[test]
fn a_convex_polygon_is_cut_into_triangles_that_cover_it() {
    let square = [
        Vector2::new(0.0, 0.0),
        Vector2::new(2.0, 0.0),
        Vector2::new(2.0, 2.0),
        Vector2::new(0.0, 2.0),
    ];

    let triangles = hit::triangulate(&square);
    assert_eq!(triangles.len(), 2);
    let covered: f32 = triangles.iter().map(|&t| triangle_area(t)).sum();
    assert!((covered - 4.0).abs() < 0.001, "covered {}", covered);
}

#[test]
fn a_concave_polygon_is_cut_without_filling_its_notch() {
    let Shape::Polygon(points) = l_shape() else {
        unreachable!()
    };

    let triangles = hit::triangulate(&points);
    assert_eq!(triangles.len(), points.len() - 2);

    let covered: f32 = triangles.iter().map(|&t| triangle_area(t)).sum();
    assert!((covered - 3.0).abs() < 0.001, "covered {}", covered);

    let notch = Vector2::new(1.5, 1.5);
    assert!(
        !triangles.iter().any(|&t| in_triangle(t, notch)),
        "the notch of an L must stay empty"
    );
}

#[test]
fn a_polygon_is_cut_the_same_whichever_way_it_was_wound() {
    let Shape::Polygon(points) = l_shape() else {
        unreachable!()
    };
    let mut reversed = points.clone();
    reversed.reverse();

    let area_of = |points: &[Vector2]| -> f32 {
        hit::triangulate(points)
            .iter()
            .map(|&t| triangle_area(t))
            .sum()
    };

    assert!(hit::signed_area(&points) * hit::signed_area(&reversed) < 0.0);
    assert!((area_of(&points) - area_of(&reversed)).abs() < 0.001);
    assert_eq!(hit::triangulate(&reversed).len(), points.len() - 2);
}

#[test]
fn a_look_is_layered_over_the_one_below_it() {
    let base = Highlight::new()
        .fill(Color::RED)
        .border(1.0, Color::BLUE)
        .opacity(0.5);
    let over = Highlight::new().fill(Color::GREEN).opacity(0.5);

    let merged = over.over(&base);
    assert_eq!(merged.fill, Some(Color::GREEN));
    assert_eq!(merged.border.map(|b| b.color), Some(Color::BLUE));
    assert_eq!(merged.opacity, Some(0.25));

    assert!(Highlight::new().is_empty());
    assert!(
        Highlight::new().opacity(0.2).is_empty(),
        "an opacity on its own draws nothing"
    );
    assert!(!Highlight::new().image("ui/glow.png").is_empty());
}

#[test]
fn a_label_sits_where_the_style_puts_it() {
    let bounds = Rectangle::new(100.0, 100.0, 200.0, 50.0);
    let size = Vector2::new(80.0, 20.0);
    let pointer = Vector2::new(400.0, 400.0);
    let style = LabelStyle::default().padding(10.0, 5.0).gap(8.0);

    let above = style.rect(size, bounds, pointer);
    assert_eq!((above.width, above.height), (100.0, 30.0));
    assert_eq!(above.x, 150.0);
    assert_eq!(above.y, 100.0 - 30.0 - 8.0);

    let below = style.clone().at(LabelAt::Below).rect(size, bounds, pointer);
    assert_eq!(below.y, 158.0);

    let center = style
        .clone()
        .at(LabelAt::Center)
        .rect(size, bounds, pointer);
    assert_eq!(center.y, 110.0);

    let at_pointer = style.at(LabelAt::Pointer).rect(size, bounds, pointer);
    assert_eq!((at_pointer.x, at_pointer.y), (408.0, 362.0));
}

#[test]
#[ignore = "opens a window; run with --ignored on a machine with a display"]
fn a_concave_shape_is_filled_up_to_its_outline() {
    let (mut rl, thread) = raylib::init().size(200, 200).title("hotspots").build();
    rl.set_trace_log(TraceLogLevel::LOG_WARNING);

    let shape = l_shape().in_image(2.0, 2.0);
    let drawn = {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        shape.draw_fill(&mut d, Rectangle::new(0.0, 0.0, 200.0, 200.0), Color::WHITE);
        unsafe { raylib::ffi::rlDrawRenderBatchActive() };
        d.load_image_from_screen(&thread)
    };

    let pixels = drawn.get_image_data();
    let at = |x: usize, y: usize| pixels[y * drawn.width() as usize + x].r;

    assert_eq!(at(50, 50), 255, "the top left arm is filled");
    assert_eq!(at(150, 50), 255, "the top right arm is filled");
    assert_eq!(at(50, 150), 255, "the bottom arm is filled");
    assert_eq!(at(150, 150), 0, "the notch of an L stays empty");
}
