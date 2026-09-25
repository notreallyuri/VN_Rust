use std::io;
use std::sync::mpsc::{self, Receiver, SyncSender};

use crate::data::assets::Assets;

pub(super) type Result<T> = io::Result<T>;

pub(super) fn error(message: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

#[derive(Debug)]
pub(super) struct VideoFrame {
    pub pts: f64,
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub aspect: f32,
    pub rgba: Vec<u8>,
}

#[derive(Debug)]
pub(super) enum Event {
    Info { sample_rate: Option<u32> },
    Video(VideoFrame),
    Audio { pts: f64, samples: Vec<f32> },
    End,
}

pub(super) struct Output(pub SyncSender<Result<Event>>);

impl Output {
    pub fn send(&self, event: Event) -> Result<()> {
        self.0
            .send(Ok(event))
            .map_err(|_| io::ErrorKind::Interrupted.into())
    }
}

pub(super) fn spawn(assets: Assets, path: String) -> io::Result<Receiver<Result<Event>>> {
    let path = path.replace('\\', "/");
    if path.is_empty()
        || std::path::Path::new(&path).components().any(|part| {
            !matches!(
                part,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
    {
        return Err(error("Video paths must be relative to the asset root"));
    }
    let (sender, receiver) = mpsc::sync_channel(4);
    std::thread::Builder::new()
        .name("vn-video".into())
        .spawn(move || {
            let output = Output(sender);
            #[cfg(feature = "video-ffmpeg")]
            let result = super::ffmpeg::decode(&assets, &path, &output);
            #[cfg(not(feature = "video-ffmpeg"))]
            let result = super::webm::decode(&assets, &path, &output);
            match result {
                Ok(()) => {
                    let _ = output.send(Event::End);
                }
                Err(e) => {
                    let _ = output.0.send(Err(e));
                }
            }
        })?;
    Ok(receiver)
}

pub(super) fn threads() -> usize {
    std::thread::available_parallelism()
        .map_or(2, |cores| cores.get().saturating_sub(1))
        .clamp(2, 4)
}

pub(super) fn dimensions(width: u32, height: u32) -> Result<usize> {
    if width == 0
        || height == 0
        || width > 4096
        || height > 4096
        || u64::from(width) * u64::from(height) > 4096 * 2160
    {
        return Err(error(
            "Video dimensions exceed the supported 4096 × 2160 pixel budget",
        ));
    }
    Ok(width as usize * height as usize * 4)
}
