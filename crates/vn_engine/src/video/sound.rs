use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use raylib::ffi;

use super::decode::{Result, error};

static ACTIVE: Mutex<Option<Pcm>> = Mutex::new(None);
static PLAYED: AtomicU64 = AtomicU64::new(0);
static CLOCK: AtomicU64 = AtomicU64::new(0);

struct Pcm {
    rate: u32,
    samples: VecDeque<f32>,
    head: u64,
    end: f64,
}

impl Pcm {
    fn new(rate: u32) -> Self {
        Self {
            rate,
            samples: VecDeque::with_capacity(rate as usize * 8),
            head: 0,
            end: 0.0,
        }
    }

    fn align(&mut self, frame: u64) {
        let discard = frame
            .saturating_sub(self.head)
            .saturating_mul(2)
            .min(self.samples.len() as u64) as usize;
        self.samples.drain(..discard);
        self.head = self.head.max(frame);
    }

    fn push(&mut self, pts: f64, samples: Vec<f32>, played: u64) -> Result<()> {
        if !pts.is_finite() || pts.abs() > 86400.0 || !samples.len().is_multiple_of(2) {
            return Err(error("Invalid video audio timestamp or sample count"));
        }
        self.align(played);
        let current = self.head + (self.samples.len() / 2) as u64;
        let mut start = (pts * self.rate as f64).round() as i64;
        if (start - current as i64).unsigned_abs() <= (self.rate / 500) as u64 {
            start = current as i64;
        }
        let count = samples.len() / 2;
        let end = start + count as i64;
        let gap = (start - current as i64).max(0) as usize;
        let trim = ((current as i64 - start).max(0) as usize).min(count);
        let extra = gap.saturating_add(count - trim).saturating_mul(2);
        if self.samples.len().saturating_add(extra) > self.rate as usize * 8 {
            return Err(error(
                "Video audio is more than four seconds ahead; remux with interleaved audio/video",
            ));
        }
        self.end = self.end.max(end as f64 / self.rate as f64);
        self.samples.extend(std::iter::repeat_n(0.0, gap * 2));
        self.samples.extend(samples.into_iter().skip(trim * 2));
        Ok(())
    }

    fn render(&mut self, frame: u64, output: &mut [f32]) {
        self.align(frame);
        for sample in output.iter_mut() {
            *sample = self.samples.pop_front().unwrap_or(0.0);
        }
        self.head += (output.len() / 2) as u64;
    }
}

unsafe extern "C" fn render(buffer: *mut std::ffi::c_void, frames: u32) {
    if buffer.is_null() {
        return;
    }
    let output =
        unsafe { std::slice::from_raw_parts_mut(buffer.cast::<f32>(), frames as usize * 2) };
    let at = PLAYED.load(Ordering::Relaxed);
    output.fill(0.0);
    if let Ok(mut active) = ACTIVE.lock()
        && let Some(pcm) = active.as_mut()
    {
        pcm.render(at, output);
    }
    PLAYED.store(at + frames as u64, Ordering::Release);
    CLOCK.store(at, Ordering::Release);
}

pub(super) struct Sound {
    stream: ffi::AudioStream,
    rate: u32,
    started: bool,
    paused: bool,
}

impl Sound {
    pub fn new(rate: u32) -> Result<Self> {
        {
            let mut active = ACTIVE.lock().map_err(error)?;
            if active.is_some() {
                return Err(error("Another video audio stream is active"));
            }
            *active = Some(Pcm::new(rate));
        }
        PLAYED.store(0, Ordering::Release);
        CLOCK.store(0, Ordering::Release);
        let stream = unsafe { ffi::LoadAudioStream(rate, 32, 2) };
        if !unsafe { ffi::IsAudioStreamValid(stream) } {
            *ACTIVE.lock().map_err(error)? = None;
            return Err(error("Could not create video audio stream"));
        }
        unsafe {
            ffi::SetAudioStreamVolume(stream, 0.0);
            ffi::SetAudioStreamCallback(stream, Some(render));
        }
        Ok(Self {
            stream,
            rate,
            started: false,
            paused: false,
        })
    }

    pub fn push(&mut self, pts: f64, samples: Vec<f32>) -> Result<()> {
        ACTIVE
            .lock()
            .map_err(error)?
            .as_mut()
            .ok_or_else(|| error("Video audio stream is closed"))?
            .push(pts, samples, PLAYED.load(Ordering::Acquire))
    }

    pub fn ready(&self) -> bool {
        ACTIVE
            .lock()
            .ok()
            .and_then(|active| {
                active
                    .as_ref()
                    .map(|pcm| pcm.samples.len() >= self.rate as usize / 5)
            })
            .unwrap_or(false)
    }

    pub fn buffered(&self) -> f64 {
        ACTIVE
            .lock()
            .ok()
            .and_then(|active| {
                active
                    .as_ref()
                    .map(|pcm| pcm.samples.len() as f64 / (2.0 * self.rate as f64))
            })
            .unwrap_or(0.0)
    }

    pub fn end(&self) -> f64 {
        ACTIVE
            .lock()
            .ok()
            .and_then(|active| active.as_ref().map(|pcm| pcm.end))
            .unwrap_or(0.0)
    }

    pub fn start(&mut self) {
        if !self.started {
            self.started = true;
            unsafe {
                ffi::PlayAudioStream(self.stream);
            }
        }
    }

    pub fn update(&mut self, volume: f32) {
        let volume = if volume.is_finite() {
            volume.clamp(0.0, 1.0)
        } else {
            1.0
        };
        unsafe {
            ffi::SetAudioStreamVolume(self.stream, volume.clamp(0.0, 1.0));
        }
    }

    pub fn clock(&self) -> f64 {
        CLOCK.load(Ordering::Acquire) as f64 / self.rate as f64
    }

    pub fn pause(&mut self, paused: bool) {
        if self.paused == paused {
            return;
        }
        if paused {
            unsafe {
                ffi::PauseAudioStream(self.stream);
            }
        } else if self.started {
            unsafe {
                ffi::ResumeAudioStream(self.stream);
            }
        }
        self.paused = paused;
    }
}

impl Drop for Sound {
    fn drop(&mut self) {
        unsafe {
            ffi::StopAudioStream(self.stream);
            ffi::UnloadAudioStream(self.stream);
        }
        if let Ok(mut active) = ACTIVE.lock() {
            *active = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires an audio device"]
    fn device_clock_stops_on_pause_and_stream_can_be_reopened() {
        let _device = raylib::audio::RaylibAudio::init_audio_device().unwrap();
        let mut sound = Sound::new(48000).unwrap();
        sound.push(0.0, vec![0.0; 48000 * 2]).unwrap();
        sound.start();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while sound.clock() == 0.0 && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(sound.clock() > 0.0);
        sound.pause(true);
        let paused = sound.clock();
        std::thread::sleep(std::time::Duration::from_millis(150));
        assert_eq!(sound.clock(), paused);
        sound.pause(false);
        while sound.clock() == paused && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(sound.clock() > paused);
        drop(sound);
        assert!(Sound::new(48000).is_ok());
    }

    #[test]
    fn audio_timestamps_insert_gaps_trim_overlaps_and_absorb_quantization() {
        let mut pcm = Pcm::new(1000);
        pcm.push(0.01, vec![1.0; 20], 0).unwrap();
        pcm.push(0.019, vec![2.0; 20], 0).unwrap();
        let mut out = [0.0; 60];
        pcm.render(0, &mut out);
        assert_eq!(&out[..20], &[0.0; 20]);
        assert_eq!(&out[20..40], &[1.0; 20]);
        assert_eq!(&out[40..], &[2.0; 20]);
        pcm.push(0.025, vec![3.0; 20], 30).unwrap();
        assert_eq!(pcm.samples.len(), 10);
    }

    #[test]
    fn audio_underrun_outputs_silence_and_discards_late_samples() {
        let mut pcm = Pcm::new(1000);
        let mut out = [1.0; 20];
        pcm.render(0, &mut out);
        assert_eq!(out, [0.0; 20]);
        pcm.push(0.0, vec![0.5; 40], 10).unwrap();
        pcm.render(10, &mut out);
        assert_eq!(out, [0.5; 20]);
        pcm.push(0.02, vec![0.75; 40], 30).unwrap();
        pcm.render(30, &mut out);
        assert_eq!(out, [0.75; 20]);
    }

    #[test]
    fn audio_queue_bounds_memory_and_rejects_invalid_timestamps() {
        let mut pcm = Pcm::new(48000);
        assert!(pcm.push(10.0, vec![0.0; 2], 0).is_err());
        for pts in [f64::NAN, f64::INFINITY, f64::MAX, -f64::MAX] {
            assert!(pcm.push(pts, vec![0.0; 2], 0).is_err());
        }
        assert!(pcm.samples.is_empty());
        assert!(pcm.push(0.0, vec![0.0], 0).is_err());
    }
}
