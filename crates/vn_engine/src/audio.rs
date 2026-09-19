use std::collections::HashMap;
use std::path::{Path, PathBuf};

use raylib::core::audio::{Music, RaylibAudio, Sound};

pub const AUDIO_EXTENSIONS: [&str; 4] = ["ogg", "mp3", "wav", "flac"];

pub fn music_path(assets: &Path, track: &str) -> Option<PathBuf> {
    find_audio(assets, "music", track)
}

pub fn sound_path(assets: &Path, id: &str) -> Option<PathBuf> {
    find_audio(assets, "sounds", id)
}

pub fn voice_path(assets: &Path, id: &str) -> Option<PathBuf> {
    find_audio(assets, "voice", id)
}

fn find_audio(assets: &Path, dir: &str, id: &str) -> Option<PathBuf> {
    AUDIO_EXTENSIONS
        .iter()
        .map(|ext| assets.join(dir).join(format!("{}.{}", id, ext)))
        .find(|path| path.is_file())
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
}

pub struct Audio {
    device: Option<&'static RaylibAudio>,
    root: PathBuf,
    config: AudioConfig,
    current: Option<Track>,
    fading: Vec<Track>,
    wanted: Option<String>,
    sounds: HashMap<String, Option<Sound<'static>>>,
    voice: Option<(String, Sound<'static>)>,
    music_volume: f32,
    sound_volume: f32,
    voice_volume: f32,
}

impl Audio {
    pub fn silent() -> Self {
        Self {
            device: None,
            root: PathBuf::new(),
            config: AudioConfig::default().enabled(false),
            current: None,
            fading: Vec::new(),
            wanted: None,
            sounds: HashMap::new(),
            voice: None,
            music_volume: 1.0,
            sound_volume: 1.0,
            voice_volume: 1.0,
        }
    }

    pub fn new(root: impl Into<PathBuf>, config: AudioConfig) -> Self {
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
            root: root.into(),
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
        let Some(path) = voice_path(&self.root, id) else {
            eprintln!(
                "⚠️ No voice file for '{}' in {}",
                id,
                self.root.join("voice").display()
            );
            return;
        };
        match device.new_sound(&path.to_string_lossy()) {
            Ok(sound) => {
                sound.set_volume(self.voice_volume);
                sound.play();
                self.voice = Some((id.to_string(), sound));
            }
            Err(e) => eprintln!("⚠️ Could not load {}: {}", path.display(), e),
        }
    }

    pub fn stop_voice(&mut self) {
        if let Some((_, voice)) = self.voice.take() {
            voice.stop();
        }
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
        let Some(path) = music_path(&self.root, track) else {
            eprintln!(
                "⚠️ No music file for '{}' in {}",
                track,
                self.root.join("music").display()
            );
            return None;
        };

        match device.new_music(&path.to_string_lossy()) {
            Ok(stream) => {
                stream.set_volume(0.0);
                stream.play_stream();
                Some(Track {
                    id: track.to_string(),
                    stream,
                    fade: Fade::fade_in(),
                })
            }
            Err(e) => {
                eprintln!("⚠️ Could not load {}: {}", path.display(), e);
                None
            }
        }
    }

    pub fn play_sound(&mut self, id: &str) {
        let Some(device) = self.device else {
            return;
        };

        let root = &self.root;
        let sound = self.sounds.entry(id.to_string()).or_insert_with(|| {
            let path = sound_path(root, id);
            let loaded = path.as_ref().and_then(|path| {
                device
                    .new_sound(&path.to_string_lossy())
                    .map_err(|e| eprintln!("⚠️ Could not load {}: {}", path.display(), e))
                    .ok()
            });
            if path.is_none() {
                eprintln!(
                    "⚠️ No sound file for '{}' in {}",
                    id,
                    root.join("sounds").display()
                );
            }
            loaded
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
