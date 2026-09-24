use std::collections::HashMap;

use raylib::core::audio::{Music, RaylibAudio, Sound};

use crate::data::assets::{Assets, extension_of};

pub const AUDIO_EXTENSIONS: [&str; 4] = ["ogg", "mp3", "wav", "flac"];

pub fn music_path(assets: &Assets, track: &str) -> Option<String> {
    find_audio(assets, "music", track)
}

pub fn sound_path(assets: &Assets, id: &str) -> Option<String> {
    find_audio(assets, "sounds", id)
}

pub fn voice_path(assets: &Assets, id: &str) -> Option<String> {
    find_audio(assets, "voice", id)
}

fn find_audio(assets: &Assets, dir: &str, id: &str) -> Option<String> {
    AUDIO_EXTENSIONS
        .iter()
        .map(|ext| format!("{}/{}.{}", dir, id, ext))
        .find(|path| assets.exists(path))
}

fn load_sound(device: &'static RaylibAudio, assets: &Assets, path: &str) -> Option<Sound<'static>> {
    let loaded = assets
        .read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| {
            device
                .new_wave_from_memory(&extension_of(path), &bytes)
                .map_err(|e| e.to_string())
        })
        .and_then(|wave| device.new_sound_from_wave(&wave).map_err(|e| e.to_string()));
    loaded
        .map_err(|e| eprintln!("⚠️ Could not load {}: {}", assets.describe(path), e))
        .ok()
}

const ENVELOPE_PER_SECOND: usize = 100;

fn load_voice(
    device: &'static RaylibAudio,
    assets: &Assets,
    path: &str,
) -> Option<(Sound<'static>, Vec<f32>)> {
    let bytes = assets
        .read(path)
        .map_err(|e| eprintln!("⚠️ Could not load {}: {}", assets.describe(path), e))
        .ok()?;
    let mut wave = device
        .new_wave_from_memory(&extension_of(path), &bytes)
        .map_err(|e| eprintln!("⚠️ Could not load {}: {}", assets.describe(path), e))
        .ok()?;
    let sound = device
        .new_sound_from_wave(&wave)
        .map_err(|e| eprintln!("⚠️ Could not load {}: {}", assets.describe(path), e))
        .ok()?;
    let rate = wave.sample_rate().max(1) as usize;
    wave.format(rate as i32, 32, 1);
    Some((sound, envelope(wave.load_samples().as_ref(), rate)))
}

fn envelope(samples: &[f32], rate: usize) -> Vec<f32> {
    let per_bucket = (rate / ENVELOPE_PER_SECOND).max(1);
    let mut levels: Vec<f32> = samples
        .chunks(per_bucket)
        .map(|bucket| {
            let sum: f32 = bucket.iter().map(|sample| sample * sample).sum();
            (sum / bucket.len() as f32).sqrt()
        })
        .collect();
    let loudest = levels.iter().copied().fold(0.0f32, f32::max);
    if loudest > 0.0 {
        for level in &mut levels {
            *level = (*level / loudest).clamp(0.0, 1.0);
        }
    }
    levels
}

#[derive(Clone, Debug, PartialEq)]
pub struct AudioConfig {
    pub enabled: bool,
    pub fade_seconds: f32,
    pub menu_music: Option<String>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fade_seconds: 1.0,
            menu_music: None,
        }
    }
}

impl AudioConfig {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn fade_seconds(mut self, seconds: f32) -> Self {
        self.fade_seconds = seconds.max(0.0);
        self
    }

    pub fn menu_music(mut self, track: impl Into<String>) -> Self {
        self.menu_music = Some(track.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fade {
    pub gain: f32,
    pub target: f32,
}

impl Fade {
    pub fn fade_in() -> Self {
        Self {
            gain: 0.0,
            target: 1.0,
        }
    }

    pub fn step(&mut self, dt: f32, seconds: f32) {
        if seconds <= 0.0 {
            self.gain = self.target;
            return;
        }
        let delta = dt / seconds;
        self.gain = if self.gain < self.target {
            (self.gain + delta).min(self.target)
        } else {
            (self.gain - delta).max(self.target)
        };
    }

    pub fn silent(&self) -> bool {
        self.target == 0.0 && self.gain <= 0.0
    }
}

struct Track {
    id: String,
    stream: Music<'static>,
    fade: Fade,
    _data: Vec<u8>,
}

pub struct Audio {
    device: Option<&'static RaylibAudio>,
    assets: Assets,
    config: AudioConfig,
    current: Option<Track>,
    fading: Vec<Track>,
    wanted: Option<String>,
    sounds: HashMap<String, Option<Sound<'static>>>,
    voice: Option<(String, Sound<'static>)>,
    voice_envelope: Vec<f32>,
    voice_started: Option<std::time::Instant>,
    music_volume: f32,
    sound_volume: f32,
    voice_volume: f32,
}

impl Audio {
    pub fn silent() -> Self {
        Self {
            device: None,
            assets: Assets::Embedded(&[]),
            config: AudioConfig::default().enabled(false),
            current: None,
            fading: Vec::new(),
            wanted: None,
            sounds: HashMap::new(),
            voice: None,
            voice_envelope: Vec::new(),
            voice_started: None,
            music_volume: 1.0,
            sound_volume: 1.0,
            voice_volume: 1.0,
        }
    }

    pub fn new(assets: impl Into<Assets>, config: AudioConfig) -> Self {
        let device = if config.enabled {
            match RaylibAudio::init_audio_device() {
                Ok(device) => Some(&*Box::leak(Box::new(device))),
                Err(e) => {
                    eprintln!("⚠️ No audio device ({:?}); the game will be silent", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            device,
            assets: assets.into(),
            config,
            ..Self::silent()
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.device.is_some()
    }

    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    pub fn music(&self) -> Option<&str> {
        self.wanted.as_deref()
    }

    pub fn set_volumes(&mut self, music: f32, sound: f32) {
        self.music_volume = music.clamp(0.0, 1.0);
        self.sound_volume = sound.clamp(0.0, 1.0);
    }

    pub fn set_voice_volume(&mut self, volume: f32) {
        self.voice_volume = volume.clamp(0.0, 1.0);
        if let Some((_, voice)) = &self.voice {
            voice.set_volume(self.voice_volume);
        }
    }

    pub fn play_voice(&mut self, id: &str) {
        self.stop_voice();
        let Some(device) = self.device else {
            return;
        };
        let Some(path) = voice_path(&self.assets, id) else {
            eprintln!(
                "⚠️ No voice file for '{}' in {}",
                id,
                self.assets.describe("voice")
            );
            return;
        };
        if let Some((sound, envelope)) = load_voice(device, &self.assets, &path) {
            sound.set_volume(self.voice_volume);
            sound.play();
            self.voice = Some((id.to_string(), sound));
            self.voice_envelope = envelope;
            self.voice_started = Some(std::time::Instant::now());
        }
    }

    pub fn stop_voice(&mut self) {
        if let Some((_, voice)) = self.voice.take() {
            voice.stop();
        }
        self.voice_envelope = Vec::new();
        self.voice_started = None;
    }

    pub fn voice_level(&self) -> f32 {
        if !self.voice_playing() {
            return 0.0;
        }
        let (Some(started), false) = (self.voice_started, self.voice_envelope.is_empty()) else {
            return 0.0;
        };
        let bucket = (started.elapsed().as_secs_f32() * ENVELOPE_PER_SECOND as f32) as usize;
        self.voice_envelope.get(bucket).copied().unwrap_or(0.0)
    }

    pub fn voice_playing(&self) -> bool {
        self.voice
            .as_ref()
            .is_some_and(|(_, voice)| voice.is_playing())
    }

    pub fn voice(&self) -> Option<&str> {
        self.voice.as_ref().map(|(id, _)| id.as_str())
    }

    pub fn play_music(&mut self, track: Option<&str>) {
        if self.wanted.as_deref() == track {
            return;
        }
        self.wanted = track.map(str::to_string);

        if let Some(mut old) = self.current.take() {
            old.fade.target = 0.0;
            self.fading.push(old);
        }

        let Some(track) = track else {
            return;
        };

        if let Some(index) = self.fading.iter().position(|t| t.id == track) {
            let mut resumed = self.fading.remove(index);
            resumed.fade.target = 1.0;
            self.current = Some(resumed);
            return;
        }

        self.current = self.load_track(track);
    }

    fn load_track(&self, track: &str) -> Option<Track> {
        let device = self.device?;
        let Some(path) = music_path(&self.assets, track) else {
            eprintln!(
                "⚠️ No music file for '{}' in {}",
                track,
                self.assets.describe("music")
            );
            return None;
        };

        let loaded = self.assets.read(&path).map(|bytes| bytes.into_owned());
        let result = loaded.map_err(|e| e.to_string()).and_then(|data| {
            device
                .new_music_from_memory(&extension_of(&path), &data)
                .map(|stream| (stream, data))
                .map_err(|e| e.to_string())
        });
        match result {
            Ok((stream, data)) => {
                stream.set_volume(0.0);
                stream.play_stream();
                Some(Track {
                    id: track.to_string(),
                    stream,
                    fade: Fade::fade_in(),
                    _data: data,
                })
            }
            Err(e) => {
                eprintln!("⚠️ Could not load {}: {}", self.assets.describe(&path), e);
                None
            }
        }
    }

    pub fn play_sound(&mut self, id: &str) {
        let Some(device) = self.device else {
            return;
        };

        let assets = &self.assets;
        let sound = self.sounds.entry(id.to_string()).or_insert_with(|| {
            let path = sound_path(assets, id);
            if path.is_none() {
                eprintln!(
                    "⚠️ No sound file for '{}' in {}",
                    id,
                    assets.describe("sounds")
                );
            }
            path.and_then(|path| load_sound(device, assets, &path))
        });

        if let Some(sound) = sound {
            sound.set_volume(self.sound_volume);
            sound.play();
        }
    }

    pub fn update(&mut self, dt: f32) {
        let seconds = self.config.fade_seconds;
        let volume = self.music_volume;

        for track in self.current.iter_mut().chain(self.fading.iter_mut()) {
            track.fade.step(dt, seconds);
            track.stream.set_volume(track.fade.gain * volume);
            track.stream.update_stream();
        }

        self.fading.retain(|track| {
            let done = track.fade.silent();
            if done {
                track.stream.stop_stream();
            }
            !done
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(seconds: f32, rate: usize, level: impl Fn(f32) -> f32) -> Vec<f32> {
        (0..(seconds * rate as f32) as usize)
            .map(|i| {
                let t = i as f32 / rate as f32;
                (t * 220.0 * std::f32::consts::TAU).sin() * level(t)
            })
            .collect()
    }

    #[test]
    fn an_envelope_is_one_level_every_ten_milliseconds() {
        let rate = 22_050;
        let levels = envelope(&tone(1.0, rate, |_| 0.5), rate);
        assert!(
            levels.len().abs_diff(ENVELOPE_PER_SECOND) <= 1,
            "a second of sound is a hundred levels, give or take the last part-bucket: {}",
            levels.len()
        );
        assert!(
            levels.iter().all(|level| (level - 1.0).abs() < 0.1),
            "an even tone stays near its own loudest, give or take how a bucket falls \
             across the wave: {levels:?}"
        );
    }

    #[test]
    fn it_follows_the_shape_of_the_sound_and_not_its_volume() {
        let rate = 22_050;
        let shape = |t: f32| if (0.2..0.4).contains(&t) { 1.0 } else { 0.0 };
        let loud = envelope(&tone(1.0, rate, shape), rate);
        let quiet = envelope(&tone(1.0, rate, |t| shape(t) * 0.05), rate);

        assert!(loud[30] > 0.9 && quiet[30] > 0.9, "the syllable is open");
        assert!(
            loud[5] < 0.05 && quiet[5] < 0.05,
            "the silence before it is shut"
        );
        assert!(
            loud.iter().zip(&quiet).all(|(a, b)| (a - b).abs() < 0.02),
            "a quiet recording opens the mouth as wide as a loud one"
        );
    }

    #[test]
    fn silence_and_scraps_do_not_panic() {
        assert!(envelope(&[], 22_050).is_empty());
        assert_eq!(envelope(&[0.0; 64], 22_050), [0.0]);
        assert_eq!(
            envelope(&[0.5; 4], 0).len(),
            4,
            "a rate of zero falls back to a level per sample rather than dividing by it"
        );
    }

    #[test]
    fn a_silent_game_has_no_voice_to_follow() {
        let audio = Audio::silent();
        assert_eq!(audio.voice_level(), 0.0);
        assert!(!audio.voice_playing());
    }
}
