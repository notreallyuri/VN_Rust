#![cfg(feature = "character-visuals")]

use std::path::PathBuf;

use novn::data::assets::Assets;
use novn::game::puppet::{BLINK, BREATH, Motion, Part, Pose, Puppet, SWAY};
use novn::game::visuals::{CharacterVisualFactory, VisualFrame};
use raylib::prelude::*;

const SIZE: (i32, i32) = (480, 640);

fn fixtures() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/puppet"
    ))
}

fn rig() -> Puppet {
    Puppet::new(600.0, 900.0)
        .folder("mary")
        .part(Part::new("body").at(300.0, 900.0).pivot(0.5, 1.0))
        .part(
            Part::new("head")
                .at(300.0, 400.0)
                .pivot(0.5, 1.0)
                .bind(SWAY, Motion::rotate(5.0))
                .bind(BREATH, Motion::offset(0.0, -6.0)),
        )
        .part(
            Part::new("eyes")
                .at(300.0, 250.0)
                .bind(BLINK, Motion::scale(1.0, 0.08))
                .bind(SWAY, Motion::offset(8.0, 0.0)),
        )
        .part(
            Part::new("mouth")
                .at(300.0, 330.0)
                .bind("mouth", Motion::scale(1.0, 2.4)),
        )
        .appearance("neutral", Pose::new())
        .appearance(
            "happy",
            Pose::new()
                .swap("mouth", "mouth_smile")
                .parameter(SWAY, 0.4),
        )
}

const HELD: [(&str, f32); 4] = [(BREATH, 0.5), (SWAY, -0.6), (BLINK, 0.75), ("mouth", 0.35)];

fn drawn(appearance: &str) -> Image {
    let (mut rl, thread) = raylib::init()
        .size(SIZE.0, SIZE.1)
        .title("puppet reference")
        .build();
    rl.set_trace_log(TraceLogLevel::LOG_WARNING);

    let assets = Assets::from(fixtures());
    let mut visual = rig()
        .load(appearance, &assets, &mut rl, &thread)
        .expect("the rig loads");
    for (id, value) in HELD {
        visual
            .set_parameter(id, Some(value))
            .expect("a bound parameter");
    }
    visual.update(0.016).expect("one frame");

    let layout = Vector2::new(SIZE.0 as f32, SIZE.1 as f32);
    let frame = VisualFrame::placed(visual.size(), layout, 0.5, Some(0.9), 1.0, 1.0);

    let mut d = rl.begin_drawing(&thread);
    d.clear_background(Color::BLACK);
    visual.draw(&mut d, frame).expect("the rig draws");
    unsafe { raylib::ffi::rlDrawRenderBatchActive() };
    d.load_image_from_screen(&thread)
}

fn difference(left: &Image, right: &Image) -> f32 {
    let (a, b) = (left.get_image_data(), right.get_image_data());
    assert_eq!(a.len(), b.len(), "the reference is a different size");
    let apart = |x: u8, y: u8| x.abs_diff(y) > 8;
    let differing = a
        .iter()
        .zip(b.iter())
        .filter(|(a, b)| apart(a.r, b.r) || apart(a.g, b.g) || apart(a.b, b.b) || apart(a.a, b.a))
        .count();
    differing as f32 / a.len() as f32
}

fn check(appearance: &str) {
    let path = fixtures().join(format!("reference/{appearance}.png"));
    let shot = drawn(appearance);

    if std::env::var_os("VN_BLESS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        shot.export_image(&path.to_string_lossy());
        println!("blessed {}", path.display());
        return;
    }

    let reference = Image::load_image(&path.to_string_lossy())
        .unwrap_or_else(|e| panic!("{}: {e}; run once with VN_BLESS=1", path.display()));
    let apart = difference(&reference, &shot);
    println!(
        "{appearance}: {:.4}% of the frame differs from its reference",
        apart * 100.0
    );
    if apart > 0.0002 {
        let beside = std::env::temp_dir().join(format!("vn-gpu-{appearance}.png"));
        shot.export_image(&beside.to_string_lossy());
        panic!(
            "{:.2}% of the frame moved against {}; this run is at {}",
            apart * 100.0,
            path.display(),
            beside.display()
        );
    }
}

#[test]
#[ignore = "opens a window; run with --ignored on a machine with a display"]
fn a_puppet_draws_the_frame_it_drew_before() {
    check("happy");
    check("neutral");
}
