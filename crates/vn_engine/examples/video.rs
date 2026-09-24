#[cfg(any(feature = "video-portable", feature = "video-ffmpeg"))]
fn main() -> Result<(), vn_engine::app::AppError> {
    use vn_engine::data::assets::EmbeddedFile;
    use vn_engine::prelude::*;
    use vn_engine::video::VideoRequest;

    static FILES: &[EmbeddedFile] = &[
        (
            "story/demo.story",
            include_bytes!("../tests/fixtures/video/story/demo.story"),
        ),
        (
            "av1-vorbis.webm",
            include_bytes!("../tests/fixtures/video/av1-vorbis.webm"),
        ),
        (
            "vp9-opus.webm",
            include_bytes!("../tests/fixtures/video/vp9-opus.webm"),
        ),
        (
            "silent.webm",
            include_bytes!("../tests/fixtures/video/silent.webm"),
        ),
        (
            "h264.mp4",
            include_bytes!("../tests/fixtures/video/h264.mp4"),
        ),
        (
            "hevc.mp4",
            include_bytes!("../tests/fixtures/video/hevc.mp4"),
        ),
    ];

    let args: Vec<_> = std::env::args().skip(1).collect();
    let smoke = args.iter().any(|arg| arg == "--smoke");
    let silent = args.iter().any(|arg| arg == "--silent");
    let clip = args
        .iter()
        .find(|arg| !arg.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| "av1-vorbis.webm".into());
    let mut app = VnApp::new("Video playback")
        .assets(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/video"))
        .schema_file(None)
        .saves_dir(std::env::temp_dir().join(format!("vn-video-example-{}", std::process::id())))
        .initial_screen(ScreenState::Playing)
        .audio(|audio| audio.enabled(!silent))
        .command_as("cutscene", move |ctx, (): ()| {
            ctx.play_video(VideoRequest::new(&clip).volume(if smoke { 0.0 } else { 1.0 }))
        })
        .command_as("finished", move |_, (): ()| {
            println!("Video screen returned to the story");
            smoke.then_some(ScreenState::Quit)
        });
    if args.iter().any(|arg| arg == "--embedded") {
        app = app
            .assets("/vn-video-embedded-assets")
            .embedded_assets(FILES);
    }
    app.run()
}

#[cfg(not(any(feature = "video-portable", feature = "video-ffmpeg")))]
fn main() {
    eprintln!("Enable video or video-ffmpeg to run this example");
}
