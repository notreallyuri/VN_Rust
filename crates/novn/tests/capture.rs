use std::fs::File;
use std::path::PathBuf;

use novn::dev::capture::{Clock, FPS, MAX_WIDTH, Recording, scaled};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("novn_capture_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn frame(size: (u16, u16), shade: u8) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(size.0 as usize * size.1 as usize * 4);
    for _ in 0..size.0 as usize * size.1 as usize {
        rgba.extend_from_slice(&[shade, 255 - shade, 40, 255]);
    }
    rgba
}

fn decode(path: &std::path::Path) -> (u16, u16, Vec<u16>) {
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = options.read_info(File::open(path).unwrap()).unwrap();
    let (width, height) = (decoder.width(), decoder.height());
    let mut delays = Vec::new();
    while let Some(frame) = decoder.read_next_frame().unwrap() {
        delays.push(frame.delay);
    }
    (width, height, delays)
}

#[test]
fn a_frame_is_scaled_down_to_the_width_limit_and_keeps_its_shape() {
    assert_eq!(scaled((1280, 720), MAX_WIDTH), (640, 360));
    assert_eq!(scaled((1920, 1080), MAX_WIDTH), (640, 360));
    assert_eq!(scaled((800, 600), MAX_WIDTH), (640, 480));
    assert_eq!(
        scaled((600, 400), MAX_WIDTH),
        (600, 400),
        "a small window is left alone"
    );
    assert_eq!(scaled((0, 0), MAX_WIDTH), (1, 1));
}

#[test]
fn delays_follow_the_clock_without_drifting() {
    let mut clock = Clock::default();
    let times = [
        0.0,
        1.0 / 15.0,
        2.0 / 15.0,
        3.0 / 15.0,
        4.0 / 15.0,
        5.0 / 15.0,
    ];
    let delays: Vec<u16> = times.windows(2).map(|w| clock.delay(w[0], w[1])).collect();
    let total: u16 = delays.iter().sum();
    assert_eq!(
        total, 33,
        "five frames at 15 fps are a third of a second: {delays:?}"
    );
    assert!(delays.iter().all(|&d| (6..=7).contains(&d)), "{delays:?}");
}

#[test]
fn a_stalled_frame_holds_for_as_long_as_it_stalled() {
    let mut clock = Clock::default();
    assert_eq!(clock.delay(0.0, 0.5), 50);
    assert_eq!(clock.delay(0.5, 0.566), 7);
}

#[test]
fn no_delay_is_shorter_than_browsers_honour() {
    let mut clock = Clock::default();
    assert_eq!(
        clock.delay(0.0, 0.001),
        2,
        "browsers slow anything under 2 cs to 10"
    );
}

#[test]
fn a_recording_is_paced_to_its_frame_rate() {
    let dir = scratch("pace");
    let recording = Recording::start(dir.join("pace.gif"), (4, 2), 10.0).unwrap();
    assert!(recording.due(10.0), "the first frame is always due");
    let mut recording = recording;
    recording.push(frame((4, 2), 0), 10.0);
    assert!(!recording.due(10.02), "too soon for the next");
    assert!(
        recording.due(10.0 + 1.0 / FPS),
        "due once a frame's time has passed"
    );
    recording.finish(10.1).wait().unwrap();
}

#[test]
fn a_recording_decodes_with_every_frame_and_its_timing() {
    let dir = scratch("decode");
    let path = dir.join("recording.gif");
    let size = (8, 4);
    let mut recording = Recording::start(&path, size, 0.0).unwrap();
    assert!(!recording.over_limit(29.0));
    assert!(recording.over_limit(30.0));

    let step = 1.0 / FPS;
    for index in 0..6 {
        recording.push(frame(size, index * 40), index as f64 * step);
    }
    let saved = recording.finish(6.0 * step).wait().unwrap();

    assert_eq!(saved.frames, 6);
    assert_eq!(saved.dropped, 0);
    let (width, height, delays) = decode(&path);
    assert_eq!((width, height), size);
    assert_eq!(delays.len(), 6);
    let total: u16 = delays.iter().sum();
    assert_eq!(total, 40, "six frames at 15 fps play for 0.4 s: {delays:?}");
}

#[test]
fn a_frame_of_the_wrong_size_is_skipped_rather_than_corrupting_the_file() {
    let dir = scratch("mixed");
    let path = dir.join("mixed.gif");
    let mut recording = Recording::start(&path, (8, 4), 0.0).unwrap();
    recording.push(frame((8, 4), 10), 0.0);
    recording.push(frame((3, 3), 20), 0.1);
    recording.push(frame((8, 4), 30), 0.2);
    let saved = recording.finish(0.3).wait().unwrap();
    assert_eq!(saved.frames, 2);
    assert_eq!(decode(&path).2.len(), 2);
}

#[test]
fn a_recording_that_cannot_open_its_file_says_so_up_front() {
    let dir = scratch("blocked");
    let blocker = dir.join("not-a-folder");
    std::fs::write(&blocker, b"").unwrap();
    assert!(Recording::start(blocker.join("x.gif"), (4, 2), 0.0).is_err());
}

fn painted(
    size: (u16, u16),
    base: [u8; 3],
    patch: Option<(u16, u16, u16, u16, [u8; 3])>,
) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(size.0 as usize * size.1 as usize * 4);
    for y in 0..size.1 {
        for x in 0..size.0 {
            let colour = match patch {
                Some((l, t, w, h, c)) if x >= l && x < l + w && y >= t && y < t + h => c,
                _ => base,
            };
            rgba.extend_from_slice(&[colour[0], colour[1], colour[2], 255]);
        }
    }
    rgba
}

struct Decoded {
    size: (u16, u16),
    frames: Vec<(u16, u16, u16, u16, u16)>,
    canvas: Vec<u8>,
}

fn replay(path: &std::path::Path) -> Decoded {
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = options.read_info(File::open(path).unwrap()).unwrap();
    let size = (decoder.width(), decoder.height());
    let mut canvas = vec![0u8; size.0 as usize * size.1 as usize * 4];
    let mut frames = Vec::new();
    while let Some(frame) = decoder.read_next_frame().unwrap() {
        frames.push((
            frame.left,
            frame.top,
            frame.width,
            frame.height,
            frame.delay,
        ));
        for y in 0..frame.height as usize {
            for x in 0..frame.width as usize {
                let from = (y * frame.width as usize + x) * 4;
                let to = ((frame.top as usize + y) * size.0 as usize + frame.left as usize + x) * 4;
                canvas[to..to + 4].copy_from_slice(&frame.buffer[from..from + 4]);
            }
        }
    }
    Decoded {
        size,
        frames,
        canvas,
    }
}

#[test]
fn a_still_screen_is_one_frame_held_for_as_long_as_it_was_still() {
    let dir = scratch("still");
    let path = dir.join("still.gif");
    let size = (16, 8);
    let mut recording = Recording::start(&path, size, 0.0).unwrap();
    for index in 0..10 {
        recording.push(painted(size, [30, 60, 90], None), index as f64 * 0.1);
    }
    let saved = recording.finish(1.0).wait().unwrap();
    assert_eq!(saved.frames, 1);
    let decoded = replay(&path);
    assert_eq!(decoded.frames.len(), 1);
    assert_eq!(
        decoded.frames[0].4, 100,
        "the one frame lasts the whole second"
    );
}

#[test]
fn a_small_change_is_encoded_as_a_small_frame_in_the_right_place() {
    let dir = scratch("region");
    let path = dir.join("region.gif");
    let size = (32, 16);
    let mut recording = Recording::start(&path, size, 0.0).unwrap();
    recording.push(painted(size, [20, 20, 20], None), 0.0);
    recording.push(
        painted(size, [20, 20, 20], Some((5, 3, 4, 2, [240, 200, 60]))),
        0.1,
    );
    recording.finish(0.2).wait().unwrap();

    let decoded = replay(&path);
    assert_eq!(decoded.frames.len(), 2);
    assert_eq!(
        decoded.frames[0].0..decoded.frames[0].2,
        0..32,
        "the first frame is whole"
    );
    let (left, top, width, height, _) = decoded.frames[1];
    assert_eq!(
        (left, top, width, height),
        (5, 3, 4, 2),
        "the second is just the patch"
    );
}

#[test]
fn replaying_the_frames_rebuilds_the_last_picture_exactly() {
    let dir = scratch("replay");
    let path = dir.join("replay.gif");
    let size = (40, 24);
    let pictures = [
        painted(size, [10, 10, 40], None),
        painted(size, [10, 10, 40], Some((0, 20, 40, 4, [200, 180, 150]))),
        painted(size, [10, 10, 40], Some((0, 20, 40, 4, [200, 180, 150]))),
        painted(size, [10, 10, 40], Some((12, 2, 9, 7, [90, 220, 120]))),
        painted(size, [10, 10, 40], Some((39, 23, 1, 1, [255, 255, 255]))),
    ];
    let mut recording = Recording::start(&path, size, 0.0).unwrap();
    for (index, picture) in pictures.iter().enumerate() {
        recording.push(picture.clone(), index as f64 * 0.1);
    }
    let saved = recording.finish(0.5).wait().unwrap();

    let decoded = replay(&path);
    assert_eq!(
        saved.frames, 4,
        "the repeated picture merged into the one before it"
    );
    assert_eq!(decoded.size, size);
    let last = pictures.last().unwrap();
    let off = decoded
        .canvas
        .chunks(4)
        .zip(last.chunks(4))
        .filter(|(got, want)| {
            got[..3]
                .iter()
                .zip(&want[..3])
                .any(|(a, b)| a.abs_diff(*b) > 8)
        })
        .count();
    assert_eq!(
        off, 0,
        "{off} pixels differ from the last picture after replay"
    );
    let total: u16 = decoded.frames.iter().map(|f| f.4).sum();
    assert_eq!(total, 50);
}

#[test]
fn a_change_is_found_in_colour_not_in_alpha() {
    use novn::dev::capture::changed;
    let size = (4, 4);
    let base = painted(size, [1, 2, 3], None);
    assert_eq!(changed(&base, &base, size), None);
    let mut alpha = base.clone();
    alpha[3] = 0;
    assert_eq!(
        changed(&base, &alpha, size),
        None,
        "alpha is forced opaque, so it is not a change"
    );
    let corner = painted(size, [1, 2, 3], Some((3, 3, 1, 1, [9, 9, 9])));
    let region = changed(&base, &corner, size).unwrap();
    assert_eq!(
        (region.left, region.top, region.width, region.height),
        (3, 3, 1, 1)
    );
}
