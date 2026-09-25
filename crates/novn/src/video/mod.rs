mod decode;
#[cfg(feature = "video-ffmpeg")]
mod ffmpeg;
mod sound;
#[cfg(all(feature = "video-portable", not(feature = "video-ffmpeg")))]
mod webm;

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, TryRecvError};

use raylib::prelude::*;

use crate::context::{DrawContext, GameContext};
use crate::screen::{Screen, ScreenState};
use crate::ui;
use decode::{Event, Result, VideoFrame, error};

const LOOKAHEAD: f64 = 0.5;
const QUEUE_BYTES: usize = 64 * 1024 * 1024;
const LOADING_LABEL_DELAY: f64 = 0.5;

#[derive(Clone, Debug)]
pub struct VideoRequest {
    pub path: String,
    pub skippable: bool,
    pub volume: f32,
    pub after: ScreenState,
}

impl VideoRequest {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            skippable: true,
            volume: 1.0,
            after: ScreenState::Playing,
        }
    }

    pub fn skippable(mut self, skippable: bool) -> Self {
        self.skippable = skippable;
        self
    }
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = if volume.is_finite() {
            volume.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self
    }
    pub fn after(mut self, screen: ScreenState) -> Self {
        self.after = screen;
        self
    }
}

pub fn backend() -> &'static str {
    if cfg!(feature = "video-ffmpeg") {
        "ffmpeg"
    } else {
        "rav1d"
    }
}

pub(crate) struct VideoScreen {
    request: Option<VideoRequest>,
    receiver: Option<Receiver<Result<Event>>>,
    frames: VecDeque<VideoFrame>,
    texture: Option<Texture2D>,
    aspect: f32,
    sound: Option<sound::Sound>,
    info: bool,
    requested_at: Option<f64>,
    ended: bool,
    started: bool,
    paused: bool,
    covered: bool,
    clock: f64,
    last_time: Option<f64>,
    last_progress: f64,
    video_end: f64,
    audio_end: f64,
}

impl VideoScreen {
    pub fn new() -> Self {
        Self {
            request: None,
            receiver: None,
            frames: VecDeque::new(),
            texture: None,
            aspect: 1.0,
            sound: None,
            info: false,
            requested_at: None,
            ended: false,
            started: false,
            paused: false,
            covered: false,
            clock: 0.0,
            last_time: None,
            last_progress: 0.0,
            video_end: 0.0,
            audio_end: 0.0,
        }
    }

    fn update_player(&mut self, ctx: &mut GameContext, now: f64) -> Result<bool> {
        if self.request.is_none() {
            let Some(request) = ctx.video_request.take() else {
                return Ok(true);
            };
            let path = request.path.clone();
            self.request = Some(request);
            self.receiver = Some(decode::spawn(ctx.resources.assets().clone(), path)?);
            self.last_progress = now;
            self.requested_at = Some(now);
        }
        if self.paused || self.covered {
            self.last_time = Some(now);
            return Ok(false);
        }
        let mut bytes: usize = self.frames.iter().map(|f| f.rgba.len()).sum();
        while bytes < QUEUE_BYTES
            && !self.ended
            && (self.queued() < LOOKAHEAD
                || self
                    .sound
                    .as_ref()
                    .is_some_and(|s| s.buffered() < LOOKAHEAD))
        {
            let event = match self.receiver.as_ref().unwrap().try_recv() {
                Ok(result) => result?,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err(error("Video decoder stopped unexpectedly"));
                }
            };
            self.last_progress = now;
            match event {
                Event::Info { sample_rate } => {
                    self.info = true;
                    if ctx.audio.is_enabled()
                        && let Some(rate) = sample_rate
                    {
                        self.sound = Some(sound::Sound::new(rate)?);
                    }
                }
                Event::Video(frame) => {
                    if !frame.pts.is_finite() {
                        return Err(error("Invalid video timestamp"));
                    }
                    self.video_end = self.video_end.max(frame.pts + frame.duration);
                    bytes += frame.rgba.len();
                    self.frames.push_back(frame);
                }
                Event::Audio { pts, samples } => {
                    if let Some(sound) = &mut self.sound {
                        sound.push(pts, samples)?;
                        self.audio_end = sound.end();
                    }
                }
                Event::End => {
                    self.ended = true;
                    self.receiver = None;
                }
            }
        }
        if !self.started
            && self.info
            && !self.frames.is_empty()
            && (self.ended
                || self.queued() >= LOOKAHEAD
                || bytes >= QUEUE_BYTES
                || self.sound.as_ref().is_none_or(|s| s.ready()))
        {
            self.started = true;
            self.last_time = Some(now);
            if let Some(sound) = &mut self.sound {
                sound.start();
            }
        }
        if self.started {
            if let Some(sound) = &mut self.sound {
                sound.update(
                    ctx.settings.values.sound_gain() * self.request.as_ref().unwrap().volume,
                );
                self.clock = sound.clock();
            } else {
                self.clock += self.last_time.map_or(0.0, |last| (now - last).max(0.0));
            }
            self.last_time = Some(now);
            let mut due = None;
            while self.frames.front().is_some_and(|f| f.pts <= self.clock) {
                due = self.frames.pop_front();
            }
            if let Some(frame) = due {
                self.upload(frame, ctx)?;
                self.last_progress = now;
            }
        }
        if self.ended && self.frames.is_empty() && self.clock >= self.video_end.max(self.audio_end)
        {
            return Ok(true);
        }
        if !self.ended
            && (!self.started || self.frames.is_empty())
            && now - self.last_progress > 15.0
        {
            return Err(error(
                "Video playback stalled; check timestamps and audio/video interleaving",
            ));
        }
        Ok(false)
    }

    fn queued(&self) -> f64 {
        match (self.frames.front(), self.frames.back()) {
            (Some(first), Some(last)) => last.pts + last.duration - first.pts,
            _ => 0.0,
        }
    }

    fn upload(&mut self, frame: VideoFrame, ctx: &mut GameContext) -> Result<()> {
        if !frame.aspect.is_finite() || frame.aspect <= 0.0 {
            return Err(error("Invalid video display aspect ratio"));
        }
        self.aspect = frame.aspect;
        if self
            .texture
            .as_ref()
            .is_none_or(|t| t.width != frame.width as i32 || t.height != frame.height as i32)
        {
            let mut image =
                Image::gen_image_color(frame.width as i32, frame.height as i32, Color::BLACK);
            image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
            let texture = ctx
                .rl
                .load_texture_from_image(ctx.thread, &image)
                .map_err(error)?;
            texture.set_texture_filter(ctx.thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            self.texture = Some(texture);
        }
        self.texture
            .as_mut()
            .unwrap()
            .update_texture(&frame.rgba)
            .map_err(error)?;
        Ok(())
    }

    fn finish(&mut self) -> ScreenState {
        self.receiver = None;
        self.sound = None;
        self.frames.clear();
        self.request
            .as_ref()
            .map_or(ScreenState::Playing, |r| r.after.clone())
    }

    fn sync_pause(&mut self, now: f64) {
        if let Some(sound) = &mut self.sound {
            sound.pause(self.paused || self.covered);
        }
        self.last_time = Some(now);
        self.last_progress = now;
    }
}

impl Screen for VideoScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        let now = ctx.rl.get_time();
        if self.request.is_some() {
            let skip = ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
                || ctx.rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                || ctx
                    .rl
                    .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                || (ctx.nav.device == crate::input::navigation::InputDevice::Gamepad
                    && (ctx.nav.accept || ctx.nav.back));
            if skip && self.request.as_ref().is_some_and(|r| r.skippable) {
                return Some(self.finish());
            }
            if ctx.rl.is_key_pressed(KeyboardKey::KEY_SPACE) || ctx.nav.pause {
                self.paused = !self.paused;
                self.sync_pause(now);
            }
        }
        match self.update_player(&mut ctx, now) {
            Ok(true) => Some(self.finish()),
            Ok(false) => None,
            Err(e) => {
                eprintln!("Video playback failed: {e}");
                ctx.notify_error(format!("Video playback failed: {e}"));
                Some(self.finish())
            }
        }
    }

    fn set_paused(&mut self, paused: bool, now: f64) {
        if self.covered != paused {
            self.covered = paused;
            self.sync_pause(now);
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let screen = ui::screen_size(d);
        d.draw_rectangle_rec(Rectangle::new(0.0, 0.0, screen.x, screen.y), Color::BLACK);
        if let Some(texture) = &self.texture {
            let height = screen.y.min(screen.x / self.aspect);
            let width = height * self.aspect;
            d.draw_texture_pro(
                texture,
                Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
                Rectangle::new(
                    (screen.x - width) * 0.5,
                    (screen.y - height) * 0.5,
                    width,
                    height,
                ),
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }
        let loading = !self.started
            && self
                .requested_at
                .is_some_and(|at| d.get_time() - at > LOADING_LABEL_DELAY);
        if loading || self.paused {
            let label = if self.paused {
                "Paused"
            } else {
                "Loading video…"
            };
            ui::draw_text_centered(
                d,
                ctx.fonts(),
                ctx.label(label),
                Vector2::new(screen.x * 0.5, screen.y * 0.9),
                &ui::TextStyle::new(ui::fonts::FontRole::Menu, 24.0, Color::WHITE),
            );
        }
    }
}

#[cfg(test)]
mod tests;
