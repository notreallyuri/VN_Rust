//! Native integration probe, deliberately separate from the story engine.
use vn_engine::data::assets::Assets;
use vn_engine::frame::target::RenderTarget;
use vn_engine::raylib::ffi;
use vn_engine::raylib::prelude::*;
use vn_live2d::{Model, ModelAssets, with_cubism};

fn screenshot(path: &str) {
    let path = std::ffi::CString::new(path).expect("VN_SHOT path");
    unsafe {
        ffi::rlDrawRenderBatchActive();
        let (width, height) = (ffi::rlGetFramebufferWidth(), ffi::rlGetFramebufferHeight());
        let pixels = ffi::rlReadScreenPixels(width, height);
        let image = ffi::Image {
            data: pixels.cast(),
            width,
            height,
            mipmaps: 1,
            format: ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
        };
        ffi::ExportImage(image, path.as_ptr());
        ffi::MemFree(pixels.cast());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("usage: viewer <asset-root> <relative.model3.json> [frame-limit]".into());
    }
    let limit = args.get(2).map(|s| s.parse::<u32>()).transpose()?;
    let assets = ModelAssets::load(&Assets::from(args[0].as_str()), &args[1])?;
    let motions: Vec<_> = assets
        .settings()
        .file_references
        .motions
        .iter()
        .flat_map(|(group, list)| (0..list.len()).map(move |i| (group.clone(), i)))
        .collect();
    let expressions: Vec<_> = assets
        .settings()
        .file_references
        .expressions
        .iter()
        .map(|e| e.name.clone())
        .collect();
    let (mut rl, thread) = vn_engine::raylib::init()
        .size(960, 720)
        .resizable()
        .title("VN_Rust / Cubism integration probe")
        .build();
    rl.set_target_fps(60);
    with_cubism(&mut rl, &thread, |sdk, rl| {
        let mut model = Model::load(sdk, rl, &thread, &assets)?;
        let mut motion = 0;
        let mut expression = 0;
        if let Some((group, index)) = motions.first() {
            model.start_motion(group, *index, true)?;
        }
        let mut target = RenderTarget::new();
        let mut frames = 0;
        let mut paused = false;
        while !rl.window_should_close() && limit.is_none_or(|limit| frames < limit) {
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                paused = !paused;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_M) && !motions.is_empty() {
                motion = (motion + 1) % motions.len();
                model.start_motion(&motions[motion].0, motions[motion].1, true)?;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_E) && !expressions.is_empty() {
                model.set_expression(&expressions[expression])?;
                expression = (expression + 1) % expressions.len();
            }
            if rl.is_key_pressed(KeyboardKey::KEY_R) {
                // Exercise model destruction/recreation without restarting the SDK.
                drop(model);
                model = Model::load(sdk, rl, &thread, &assets)?;
            }
            if !paused {
                model.update(rl.get_frame_time())?;
            }
            let size = (rl.get_screen_width().max(1), rl.get_screen_height().max(1));
            target.resize(rl, &thread, size);
            let aspect = size.1 as f32 / size.0 as f32;
            let matrix = [
                aspect, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ];
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);
            if let Some(frame) = target.frame_mut() {
                let mut t = d.begin_texture_mode(&thread, frame);
                t.clear_background(Color::new(30, 35, 45, 255));
                t.draw_rectangle(0, 0, 24, 24, Color::RED);
                model.draw(&mut t, matrix, (size.0 as u32, size.1 as u32), 1.0)?;
                t.draw_rectangle(size.0 - 24, 0, 24, 24, Color::GREEN);
            }
            if let Some(frame) = target.frame() {
                d.draw_texture_pro(
                    frame.texture(),
                    target.source(),
                    Rectangle::new(0.0, 0.0, size.0 as f32, size.1 as f32),
                    Vector2::zero(),
                    0.0,
                    Color::WHITE,
                );
            }
            d.draw_text(
                "M: motion | E: expression | Space: pause | R: recreate",
                28,
                12,
                18,
                Color::WHITE,
            );
            frames += 1;
            if let Ok(shot) = std::env::var("VN_SHOT")
                && frames == 120
            {
                screenshot(&shot);
            }
        }
        Ok(())
    })?;
    Ok(())
}
