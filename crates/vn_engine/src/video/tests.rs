use std::path::PathBuf;
use std::time::Duration;

use super::decode::{Event, spawn};
use crate::data::assets::{Assets, EmbeddedFile};

static FILES: &[EmbeddedFile] = &[
    (
        "av1-vorbis.webm",
        include_bytes!("../../tests/fixtures/video/av1-vorbis.webm"),
    ),
    (
        "silent.webm",
        include_bytes!("../../tests/fixtures/video/silent.webm"),
    ),
    (
        "vp9-opus.webm",
        include_bytes!("../../tests/fixtures/video/vp9-opus.webm"),
    ),
    (
        "h264.mp4",
        include_bytes!("../../tests/fixtures/video/h264.mp4"),
    ),
    (
        "hevc.mp4",
        include_bytes!("../../tests/fixtures/video/hevc.mp4"),
    ),
    (
        "misnamed.webm",
        include_bytes!("../../tests/fixtures/video/h264.mp4"),
    ),
    ("broken.webm", b"not a movie"),
];

fn events(assets: Assets, path: &str) -> Result<Vec<Event>, String> {
    let receiver = spawn(assets, path.into()).map_err(|e| e.to_string())?;
    let mut events = Vec::new();
    loop {
        let event = receiver
            .recv_timeout(Duration::from_secs(10))
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        let ended = matches!(event, Event::End);
        events.push(event);
        if ended {
            return Ok(events);
        }
    }
}

#[test]
fn video_decodes_embedded_and_folder_assets_with_audio_and_delayed_frames() {
    for assets in [
        Assets::Embedded(FILES),
        Assets::Dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/video")),
    ] {
        let decoded = events(assets, "av1-vorbis.webm").unwrap();
        assert!(matches!(
            decoded[0],
            Event::Info {
                sample_rate: Some(48000)
            }
        ));
        let frames: Vec<_> = decoded
            .iter()
            .filter_map(|e| {
                if let Event::Video(f) = e {
                    Some(f)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(frames.len(), 12);
        assert!(frames.windows(2).all(|w| w[0].pts < w[1].pts));
        assert!(frames[0].pts.abs() < 0.05);
        assert!(frames.last().unwrap().pts > 0.9);
        for frame in frames {
            assert_eq!((frame.width, frame.height), (96, 64));
            assert_eq!(frame.rgba.len(), 96 * 64 * 4);
            assert!(frame.rgba.as_chunks::<4>().0.iter().all(|p| p[3] == 255));
            assert!(
                frame
                    .rgba
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|p| p[0] > 150 && p[1] < 80)
            );
        }
        let samples: Vec<f32> = decoded
            .into_iter()
            .filter_map(|e| {
                if let Event::Audio { samples, .. } = e {
                    Some(samples)
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        assert!(
            (94000..=100000).contains(&samples.len()),
            "{} samples",
            samples.len()
        );
        assert!(samples.iter().all(|s| s.is_finite()));
        assert!(samples.iter().any(|s| s.abs() > 0.01));
    }
}

#[test]
fn video_decodes_silent_clip_and_reports_missing_or_malformed_assets() {
    let decoded = events(Assets::Embedded(FILES), "silent.webm").unwrap();
    assert!(matches!(decoded[0], Event::Info { sample_rate: None }));
    assert!(!decoded.iter().any(|e| matches!(e, Event::Audio { .. })));
    assert!(events(Assets::Embedded(FILES), "missing.webm").is_err());
    assert!(events(Assets::Embedded(FILES), "broken.webm").is_err());
}

#[test]
fn video_desktop_rejects_h264_and_hevc_files() {
    for path in ["h264.mp4", "hevc.mp4", "misnamed.webm"] {
        let failure = events(Assets::Embedded(FILES), path).unwrap_err();
        #[cfg(feature = "video-ffmpeg")]
        assert!(failure.contains("disabled on desktop"), "{failure}");
        #[cfg(not(feature = "video-ffmpeg"))]
        assert!(!failure.is_empty());
    }
}

#[test]
fn video_backend_selection_and_format_support() {
    #[cfg(feature = "video-ffmpeg")]
    {
        assert_eq!(super::backend(), "ffmpeg");
        let decoded = events(Assets::Embedded(FILES), "vp9-opus.webm").unwrap();
        assert_eq!(
            decoded
                .iter()
                .filter(|e| matches!(e, Event::Video(_)))
                .count(),
            12
        );
        assert!(decoded.iter().any(|e| matches!(e, Event::Audio { .. })));
    }
    #[cfg(not(feature = "video-ffmpeg"))]
    {
        assert_eq!(super::backend(), "rav1d");
        let failure = events(Assets::Embedded(FILES), "vp9-opus.webm").unwrap_err();
        assert!(failure.contains("AV1"), "{failure}");
    }
}

#[test]
fn video_worker_exits_when_a_full_queue_is_cancelled() {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let output = super::decode::Output(sender);
    let (done, completed) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        #[cfg(feature = "video-ffmpeg")]
        let result = super::ffmpeg::decode(&Assets::Embedded(FILES), "av1-vorbis.webm", &output);
        #[cfg(not(feature = "video-ffmpeg"))]
        let result = super::webm::decode(&Assets::Embedded(FILES), "av1-vorbis.webm", &output);
        done.send(result.map_err(|e| e.kind())).unwrap();
    });
    assert!(matches!(
        receiver
            .recv_timeout(Duration::from_secs(10))
            .unwrap()
            .unwrap(),
        Event::Info { .. }
    ));
    drop(receiver);
    assert_eq!(
        completed.recv_timeout(Duration::from_secs(10)).unwrap(),
        Err(std::io::ErrorKind::Interrupted)
    );
    worker.join().unwrap();
}

#[test]
fn video_rejects_paths_outside_the_asset_root() {
    for path in ["../movie.webm", "/movie.webm", ""] {
        assert!(spawn(Assets::Embedded(FILES), path.into()).is_err());
    }
}
