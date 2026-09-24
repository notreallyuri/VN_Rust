#![cfg(feature = "engine")]

use std::path::PathBuf;

use vn_engine::data::assets::Assets;
use vn_engine::game::visuals::{CharacterVisualFactory, VisualFrame};
use vn_engine::raylib::prelude::*;
use vn_live2d::{Appearance, Live2dCharacter};

const SIZE: (i32, i32) = (480, 640);
const HELD: [(&str, f32); 3] = [
    ("ParamAngleZ", 12.0),
    ("ParamEyeLOpen", 0.2),
    ("ParamEyeROpen", 0.2),
];

fn model() -> Option<(PathBuf, String)> {
    let root = PathBuf::from(std::env::var_os("CUBISM_SDK_ROOT")?);
    let dir = root.join("Samples/Resources/Hiyori");
    dir.join("Hiyori.model3.json")
        .is_file()
        .then(|| (dir, "Hiyori.model3.json".to_string()))
}

fn baseline(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/live2d-reference")
        .join(format!("{name}.png"))
}

fn difference(left: &Image, right: &Image) -> f32 {
    let (a, b) = (left.get_image_data(), right.get_image_data());
    assert_eq!(a.len(), b.len(), "the baseline is a different size");
    let apart = |x: u8, y: u8| x.abs_diff(y) > 8;
    let differing = a
        .iter()
        .zip(b.iter())
        .filter(|(a, b)| apart(a.r, b.r) || apart(a.g, b.g) || apart(a.b, b.b) || apart(a.a, b.a))
        .count();
    differing as f32 / a.len() as f32
}

#[test]
#[ignore = "needs the Cubism SDK, a model and a display; run with --ignored"]
fn a_model_draws_the_frame_it_drew_before() {
    let Some((dir, file)) = model() else {
        println!("CUBISM_SDK_ROOT is not set, or Hiyori is not in its samples; nothing to compare");
        return;
    };

    let (mut rl, thread) = vn_engine::raylib::init()
        .size(SIZE.0, SIZE.1)
        .title("model reference")
        .build();
    rl.set_trace_log(TraceLogLevel::LOG_WARNING);

    let character = Live2dCharacter::new(format!("characters/one/{file}"))
        .appearance("neutral", Appearance::new().motion("Idle", 0, false));
    let root = std::env::temp_dir().join(format!("vn-live2d-gpu-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("characters")).unwrap();
    std::os::unix::fs::symlink(&dir, root.join("characters/one")).unwrap();
    let assets = Assets::from(root.clone());

    let mut visual = character
        .load("neutral", &assets, &mut rl, &thread)
        .expect("the model loads");
    for (id, value) in HELD {
        visual.set_parameter(id, Some(value)).expect("a parameter");
    }
    visual.update(0.0).expect("one frame");

    let layout = Vector2::new(SIZE.0 as f32, SIZE.1 as f32);
    let frame = VisualFrame::placed(visual.size(), layout, 0.5, Some(0.9), 1.0, 1.0);
    let shot = {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        visual.draw(&mut d, frame).expect("the model draws");
        d.load_image_from_screen(&thread)
    };
    drop(visual);
    let _ = std::fs::remove_dir_all(&root);

    let path = baseline("hiyori-neutral");
    if !path.is_file() || std::env::var_os("VN_BLESS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        shot.export_image(&path.to_string_lossy());
        println!("baseline written to {}", path.display());
        return;
    }

    let reference = Image::load_image(&path.to_string_lossy()).expect("the baseline loads");
    let apart = difference(&reference, &shot);
    println!(
        "{:.4}% of the frame differs from the baseline",
        apart * 100.0
    );
    assert!(apart <= 0.0002, "{:.2}% of the frame moved", apart * 100.0);
}
