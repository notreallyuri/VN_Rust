use vn_engine::target::destination;

#[test]
fn a_matching_screen_uses_every_pixel() {
    let rect = destination((1280, 720), (1280, 720));
    assert_eq!((rect.x, rect.y), (0.0, 0.0));
    assert_eq!((rect.width, rect.height), (1280.0, 720.0));
}

#[test]
fn a_wider_screen_gets_pillarboxed() {
    let rect = destination((1280, 720), (2560, 720));
    assert_eq!((rect.width, rect.height), (1280.0, 720.0));
    assert_eq!((rect.x, rect.y), (640.0, 0.0));
}

#[test]
fn a_taller_screen_gets_letterboxed() {
    let rect = destination((1280, 720), (1280, 1440));
    assert_eq!((rect.width, rect.height), (1280.0, 720.0));
    assert_eq!((rect.x, rect.y), (0.0, 360.0));
}

#[test]
fn a_bigger_screen_scales_up_and_keeps_the_aspect_ratio() {
    let rect = destination((1280, 720), (1920, 1080));
    assert_eq!((rect.width, rect.height), (1920.0, 1080.0));
    assert_eq!((rect.x, rect.y), (0.0, 0.0));
}

#[test]
fn a_target_that_could_not_be_created_falls_back_to_the_whole_screen() {
    let rect = destination((0, 0), (800, 600));
    assert_eq!((rect.width, rect.height), (800.0, 600.0));
}

#[test]
#[ignore = "opens a window; run with --ignored on a machine with a display"]
fn the_screen_capture_reads_the_render_target() {
    use raylib::prelude::*;
    use vn_engine::target::RenderTarget;

    let (mut rl, thread) = raylib::init().size(320, 200).title("render target").build();
    rl.set_trace_log(TraceLogLevel::LOG_WARNING);

    let mut target = RenderTarget::new();
    target.resize(&mut rl, &thread, (320, 200));
    assert!(target.frame().is_some(), "no render target");

    let mut d = rl.begin_drawing(&thread);
    let drawn = {
        let mut t = d.begin_texture_mode(&thread, target.frame_mut().unwrap());
        t.clear_background(Color::GREEN);
        t.draw_rectangle(0, 0, 160, 200, Color::BLUE);
        unsafe { raylib::ffi::rlDrawRenderBatchActive() };
        t.load_image_from_screen(&thread)
    };

    assert_eq!((drawn.width(), drawn.height()), (320, 200));
    let pixels = drawn.get_image_data();
    let left = pixels[0];
    let right = pixels[320 * 100 + 300];
    assert_eq!(
        ((left.r, left.g, left.b), (right.r, right.g, right.b)),
        (
            (Color::BLUE.r, Color::BLUE.g, Color::BLUE.b),
            (Color::GREEN.r, Color::GREEN.g, Color::GREEN.b)
        ),
        "a capture inside texture mode must read the render target, or save thumbnails break"
    );
}
