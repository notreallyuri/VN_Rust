use std::fs::File;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::thread::{self, JoinHandle};

pub const FPS: f64 = 15.0;
pub const MAX_WIDTH: i32 = 640;
pub const MAX_SECONDS: f64 = 30.0;
pub const QUEUE: usize = 48;
pub const SPEED: i32 = 10;
const MIN_DELAY: u16 = 2;

pub fn scaled(size: (i32, i32), max_width: i32) -> (u16, u16) {
    let (width, height) = (size.0.max(1), size.1.max(1));
    if width <= max_width {
        return (
            width.min(u16::MAX as i32) as u16,
            height.min(u16::MAX as i32) as u16,
        );
    }
    let h = ((height as f64 * max_width as f64 / width as f64).round() as i32).max(1);
    (max_width as u16, h.min(u16::MAX as i32) as u16)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Clock {
    start: Option<u64>,
    sent: u64,
}

impl Clock {
    pub fn delay(&mut self, from: f64, to: f64) -> u16 {
        let start = *self
            .start
            .get_or_insert((from * 100.0).round().max(0.0) as u64);
        let target = ((to * 100.0).round().max(0.0) as u64).saturating_sub(start);
        let delay = target.saturating_sub(self.sent).max(MIN_DELAY as u64);
        self.sent += delay;
        delay.min(u16::MAX as u64) as u16
    }
}

enum Message {
    Frame { rgba: Vec<u8>, delay: u16 },
    Finish,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Saved {
    pub path: PathBuf,
    pub frames: usize,
    pub dropped: usize,
}

pub struct Recording {
    path: PathBuf,
    size: (u16, u16),
    sender: SyncSender<Message>,
    worker: JoinHandle<io::Result<usize>>,
    started: f64,
    held: Option<(Vec<u8>, f64)>,
    clock: Clock,
    dropped: usize,
}

impl Recording {
    pub fn start(path: impl Into<PathBuf>, size: (u16, u16), now: f64) -> io::Result<Self> {
        let path = path.into();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let file = File::create(&path)?;
        let (sender, receiver) = sync_channel(QUEUE);
        let worker = thread::Builder::new()
            .name("novn-gif".into())
            .spawn(move || encode(file, size, receiver))?;
        Ok(Self {
            path,
            size,
            sender,
            worker,
            started: now,
            held: None,
            clock: Clock::default(),
            dropped: 0,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    pub fn elapsed(&self, now: f64) -> f64 {
        (now - self.started).max(0.0)
    }

    pub fn over_limit(&self, now: f64) -> bool {
        self.elapsed(now) >= MAX_SECONDS
    }

    pub fn due(&self, now: f64) -> bool {
        match &self.held {
            None => true,
            Some((_, at)) => now - at >= 1.0 / FPS - 1e-6,
        }
    }

    pub fn dropped(&self) -> usize {
        self.dropped
    }

    pub fn push(&mut self, rgba: Vec<u8>, now: f64) {
        if let Some((previous, at)) = self.held.take() {
            let delay = self.clock.delay(at, now);
            self.send(previous, delay);
        }
        self.held = Some((rgba, now));
    }

    fn send(&mut self, rgba: Vec<u8>, delay: u16) {
        match self.sender.try_send(Message::Frame { rgba, delay }) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => self.dropped += 1,
            Err(TrySendError::Disconnected(_)) => self.dropped += 1,
        }
    }

    pub fn finish(mut self, now: f64) -> Finishing {
        if let Some((last, at)) = self.held.take() {
            let delay = self.clock.delay(at, now.max(at + 1.0 / FPS));
            let _ = self.sender.send(Message::Frame { rgba: last, delay });
        }
        let _ = self.sender.send(Message::Finish);
        Finishing {
            path: self.path,
            worker: Some(self.worker),
            dropped: self.dropped,
        }
    }
}

pub struct Finishing {
    path: PathBuf,
    worker: Option<JoinHandle<io::Result<usize>>>,
    dropped: usize,
}

impl Finishing {
    pub fn poll(&mut self) -> Option<io::Result<Saved>> {
        if self.worker.as_ref().is_some_and(|w| !w.is_finished()) {
            return None;
        }
        Some(self.wait())
    }

    pub fn wait(&mut self) -> io::Result<Saved> {
        let worker = self
            .worker
            .take()
            .ok_or_else(|| io::Error::other("already collected"))?;
        let frames = worker
            .join()
            .map_err(|_| io::Error::other("the GIF encoder panicked"))??;
        Ok(Saved {
            path: self.path.clone(),
            frames,
            dropped: self.dropped,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub left: u16,
    pub top: u16,
    pub width: u16,
    pub height: u16,
}

pub fn changed(previous: &[u8], next: &[u8], size: (u16, u16)) -> Option<Region> {
    let (width, height) = (size.0 as usize, size.1 as usize);
    let (mut left, mut top, mut right, mut bottom) = (width, height, 0, 0);
    for y in 0..height {
        let row = y * width * 4;
        for x in 0..width {
            let at = row + x * 4;
            if previous[at..at + 3] != next[at..at + 3] {
                left = left.min(x);
                right = right.max(x);
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
    }
    (left <= right).then(|| Region {
        left: left as u16,
        top: top as u16,
        width: (right - left + 1) as u16,
        height: (bottom - top + 1) as u16,
    })
}

pub fn crop(rgba: &[u8], size: (u16, u16), region: Region) -> Vec<u8> {
    let stride = size.0 as usize * 4;
    let mut out = Vec::with_capacity(region.width as usize * region.height as usize * 4);
    for y in region.top as usize..(region.top + region.height) as usize {
        let start = y * stride + region.left as usize * 4;
        out.extend_from_slice(&rgba[start..start + region.width as usize * 4]);
    }
    out
}

struct Pending {
    region: Region,
    rgba: Vec<u8>,
    delay: u16,
}

fn write<W: io::Write>(encoder: &mut gif::Encoder<W>, pending: Pending) -> io::Result<()> {
    let Pending {
        region,
        mut rgba,
        delay,
    } = pending;
    let mut frame = gif::Frame::from_rgba_speed(region.width, region.height, &mut rgba, SPEED);
    frame.left = region.left;
    frame.top = region.top;
    frame.delay = delay;
    frame.dispose = gif::DisposalMethod::Keep;
    encoder
        .write_frame(&frame)
        .map_err(|e| io::Error::other(e.to_string()))
}

fn encode(file: File, size: (u16, u16), receiver: Receiver<Message>) -> io::Result<usize> {
    let failed = |e: gif::EncodingError| io::Error::other(e.to_string());
    let mut encoder =
        gif::Encoder::new(BufWriter::new(file), size.0, size.1, &[]).map_err(failed)?;
    encoder.set_repeat(gif::Repeat::Infinite).map_err(failed)?;
    let expected = size.0 as usize * size.1 as usize * 4;
    let whole = Region {
        left: 0,
        top: 0,
        width: size.0,
        height: size.1,
    };
    let mut previous: Option<Vec<u8>> = None;
    let mut pending: Option<Pending> = None;
    let mut frames = 0;
    for message in receiver {
        let (rgba, delay) = match message {
            Message::Frame { rgba, delay } if rgba.len() == expected => (rgba, delay),
            Message::Frame { .. } => continue,
            Message::Finish => break,
        };
        let region = match &previous {
            None => Some(whole),
            Some(previous) => changed(previous, &rgba, size),
        };
        match region {
            None => {
                if let Some(pending) = &mut pending {
                    pending.delay = pending.delay.saturating_add(delay);
                }
            }
            Some(region) => {
                if let Some(ready) = pending.take() {
                    write(&mut encoder, ready)?;
                    frames += 1;
                }
                pending = Some(Pending {
                    region,
                    rgba: crop(&rgba, size, region),
                    delay,
                });
                previous = Some(rgba);
            }
        }
    }
    if let Some(ready) = pending.take() {
        write(&mut encoder, ready)?;
        frames += 1;
    }
    let mut writer = encoder.into_inner().map_err(failed)?;
    io::Write::flush(&mut writer)?;
    Ok(frames)
}
