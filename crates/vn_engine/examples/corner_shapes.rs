use vn_engine::raylib::prelude::*;
use vn_engine::{CornerShape, Corners, GradientDirection, PanelStyle};

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1000, 520)
        .title("Corner shapes")
        .msaa_4x()
        .build();
    let shapes = [
        ("square", Corners::square()),
        ("round", Corners::round(18.0)),
        ("bevel", Corners::bevel(18.0)),
        ("scoop", Corners::scoop(18.0)),
        ("notch", Corners::notch(14.0)),
    ];
    let mixed = Corners::scoop(22.0)
        .top_right(CornerShape::Bevel, 22.0)
        .bottom_left(CornerShape::Round, 22.0)
        .bottom_right(CornerShape::Notch, 16.0);
    let shot = std::env::args().nth(1);
    let mut frame = 0;
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(30, 24, 20, 255));
        for (i, (name, corners)) in shapes.iter().enumerate() {
            let x = 30.0 + i as f32 * 192.0;
            let panel = PanelStyle::new(Color::new(60, 48, 38, 255))
                .corners(*corners)
                .border(2.0, Color::new(200, 170, 110, 255));
            panel.draw(&mut d, Rectangle::new(x, 30.0, 170.0, 110.0));
            let framed = PanelStyle::new(Color::new(22, 18, 15, 240))
                .corners(*corners)
                .border(1.0, Color::new(150, 120, 80, 255))
                .inner_border(5.0, 1.0, Color::new(150, 120, 80, 140))
                .shadow(0.0, 6.0, Color::new(0, 0, 0, 140));
            framed.draw(&mut d, Rectangle::new(x, 170.0, 170.0, 110.0));
            d.draw_text(name, x as i32 + 8, 290, 20, Color::RAYWHITE);
        }
        PanelStyle::new(Color::new(20, 16, 13, 250))
            .gradient(Color::new(20, 16, 13, 0), GradientDirection::Horizontal)
            .corners(mixed)
            .border(3.0, Color::new(200, 170, 110, 255))
            .draw(&mut d, Rectangle::new(30.0, 330.0, 940.0, 160.0));
        drop(d);
        frame += 1;
        if frame == 5
            && let Some(path) = &shot
        {
            rl.take_screenshot(&thread, path);
            break;
        }
    }
}
