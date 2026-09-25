use novn::raylib::prelude::*;
use novn::ui::shape::{Corner, CornerShape, Corners, GradientDirection, PanelStyle};

fn rect() -> Rectangle {
    Rectangle::new(10.0, 20.0, 200.0, 100.0)
}

fn close(a: Vector2, b: (f32, f32)) -> bool {
    (a.x - b.0).abs() < 0.01 && (a.y - b.1).abs() < 0.01
}

#[test]
fn square_corners_are_the_rectangle() {
    let points = Corners::square().outline(rect());
    assert_eq!(points.len(), 4);
    for (point, expected) in
        points
            .iter()
            .zip([(10.0, 20.0), (210.0, 20.0), (210.0, 120.0), (10.0, 120.0)])
    {
        assert!(close(*point, expected), "{:?} != {:?}", point, expected);
    }
}

#[test]
fn bevels_cut_each_corner_diagonally() {
    let points = Corners::bevel(10.0).outline(rect());
    assert_eq!(points.len(), 8);
    assert!(close(points[0], (10.0, 30.0)));
    assert!(close(points[1], (20.0, 20.0)));
    assert!(close(points[2], (200.0, 20.0)));
    assert!(close(points[3], (210.0, 30.0)));
}

#[test]
fn notches_step_inwards() {
    let points = Corners::notch(10.0).outline(rect());
    assert_eq!(points.len(), 12);
    assert!(close(points[0], (10.0, 30.0)));
    assert!(close(points[1], (20.0, 30.0)));
    assert!(close(points[2], (20.0, 20.0)));
}

#[test]
fn rounds_and_scoops_start_and_end_on_the_edges() {
    for corners in [Corners::round(12.0), Corners::scoop(12.0)] {
        let points = corners.outline(rect());
        let first = points[0];
        assert!(close(first, (10.0, 32.0)), "{:?}", first);
        let last_of_first_corner = points[points.len() / 4 - 1];
        assert!(
            close(last_of_first_corner, (22.0, 20.0)),
            "{:?}",
            last_of_first_corner
        );
    }
}

#[test]
fn rounds_bulge_out_and_scoops_bite_in() {
    let r = rect();
    let near_corner = Vector2::new(13.0, 23.0);
    let past_the_curve = Vector2::new(21.0, 29.0);

    let round = Corners::round(12.0);
    assert!(!novn::ui::shape::contains(r, &round, near_corner));
    assert!(novn::ui::shape::contains(r, &round, past_the_curve));

    let scoop = Corners::scoop(12.0);
    assert!(!novn::ui::shape::contains(r, &scoop, near_corner));
    assert!(!novn::ui::shape::contains(
        r,
        &scoop,
        Vector2::new(18.0, 26.0)
    ));
    assert!(novn::ui::shape::contains(
        r,
        &scoop,
        Vector2::new(24.0, 34.0)
    ));
    assert!(novn::ui::shape::contains(
        r,
        &scoop,
        Vector2::new(110.0, 70.0)
    ));
}

#[test]
fn sizes_are_clamped_to_half_the_short_side() {
    let corners = Corners::bevel(500.0).resolved(rect());
    assert!(corners.iter().all(|c| c.size == 50.0));
}

#[test]
fn roundness_is_relative_to_the_short_side() {
    let corners = Corners::roundness(0.5).resolved(rect());
    assert!(
        corners
            .iter()
            .all(|c| c.size == 25.0 && c.shape == CornerShape::Round)
    );
}

#[test]
fn each_corner_can_differ() {
    let corners = Corners::scoop(10.0)
        .top_right(CornerShape::Bevel, 6.0)
        .bottom(CornerShape::Square, 0.0);
    assert_eq!(corners.top_left, Corner::new(CornerShape::Scoop, 10.0));
    assert_eq!(corners.top_right, Corner::new(CornerShape::Bevel, 6.0));
    assert_eq!(corners.bottom_left, Corner::SQUARE);
    assert_eq!(corners.bottom_right, Corner::SQUARE);
    assert!(!corners.is_square());
    assert!(Corners::round(0.0).is_square());
    assert_eq!(Corners::from(CornerShape::Bevel), Corners::bevel(8.0));
}

#[test]
fn panel_style_builder() {
    let panel = PanelStyle::new(Color::BLACK)
        .corners(Corners::scoop(14.0))
        .border(1.0, Color::GOLD)
        .inner_border(5.0, 1.0, Color::BROWN)
        .gradient(Color::BLANK, GradientDirection::Vertical)
        .shadow(0.0, 6.0, Color::BLACK);

    assert_eq!(panel.corners, Corners::scoop(14.0));
    assert_eq!(panel.border.map(|b| b.width), Some(1.0));
    assert_eq!(panel.inner_border.map(|(inset, _)| inset), Some(5.0));
    assert_eq!(
        panel.gradient.map(|g| g.direction),
        Some(GradientDirection::Vertical)
    );
    assert!(panel.no_border().border.is_none());
    assert_eq!(
        PanelStyle::new(Color::BLACK).roundness(0.2).corners,
        Corners::roundness(0.2)
    );
}

#[test]
fn dialogue_box_margins_bottom_and_max_width() {
    use novn::screens::playing::DialogueBoxStyle;
    let screen = Vector2::new(1280.0, 720.0);

    let default = DialogueBoxStyle::default().rect(screen);
    assert_eq!(
        (default.x, default.y, default.width, default.height),
        (40.0, 510.0, 1200.0, 170.0)
    );

    let styled = DialogueBoxStyle::default()
        .height(150.0)
        .margin(56.0)
        .bottom(52.0)
        .max_width(1120.0)
        .rect(screen);
    assert_eq!(
        (styled.x, styled.y, styled.width, styled.height),
        (80.0, 518.0, 1120.0, 150.0)
    );

    let narrow = DialogueBoxStyle::default()
        .max_width(2000.0)
        .rect(Vector2::new(800.0, 600.0));
    assert_eq!((narrow.x, narrow.width), (40.0, 720.0));
}

#[test]
fn name_plates_sit_on_the_top_edge() {
    use novn::screens::playing::NamePlate;
    let dialogue = Rectangle::new(80.0, 518.0, 1120.0, 150.0);
    let plate = NamePlate::default()
        .padding(20.0, 6.0)
        .indent(30.0)
        .overlap(14.0)
        .min_width(150.0);

    let short = plate.rect(dialogue, Vector2::new(60.0, 24.0));
    assert_eq!(
        (short.x, short.y, short.width, short.height),
        (110.0, 518.0 - 36.0 + 14.0, 150.0, 36.0)
    );
    let long = plate.rect(dialogue, Vector2::new(200.0, 24.0));
    assert_eq!(long.width, 240.0);
}
