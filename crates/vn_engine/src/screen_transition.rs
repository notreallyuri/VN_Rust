use crate::ease::{Easing, Tween};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScreenTransitionKind {
    None,
    #[default]
    Crossfade,
    Fade,
    SlideLeft,
    SlideRight,
}

#[derive(Debug, Clone, Copy)]
pub struct ScreenTransitionConfig {
    pub kind: ScreenTransitionKind,
    pub seconds: f32,
}

impl Default for ScreenTransitionConfig {
    fn default() -> Self {
        Self {
            kind: ScreenTransitionKind::Crossfade,
            seconds: 0.2,
        }
    }
}

impl ScreenTransitionConfig {
    pub fn new(kind: ScreenTransitionKind, seconds: f32) -> Self {
        Self { kind, seconds }
    }

    pub fn none() -> Self {
        Self::new(ScreenTransitionKind::None, 0.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScreenTransition {
    kind: ScreenTransitionKind,
    tween: Tween,
}

impl ScreenTransition {
    pub fn start(config: ScreenTransitionConfig, now: f64) -> Option<Self> {
        if config.kind == ScreenTransitionKind::None || config.seconds <= 0.0 {
            return None;
        }
        Some(Self {
            kind: config.kind,
            tween: Tween::new(now, config.seconds, Easing::Smooth),
        })
    }

    pub fn kind(&self) -> ScreenTransitionKind {
        self.kind
    }

    pub fn finished(&self, now: f64) -> bool {
        self.tween.finished(now)
    }

    pub fn previous_alpha(&self, now: f64) -> f32 {
        match self.kind {
            ScreenTransitionKind::Crossfade => 1.0 - self.tween.progress(now),
            ScreenTransitionKind::Fade => {
                if self.tween.elapsed(now) < 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
            ScreenTransitionKind::SlideLeft | ScreenTransitionKind::SlideRight => 1.0,
            ScreenTransitionKind::None => 0.0,
        }
    }

    pub fn cover_alpha(&self, now: f64) -> f32 {
        match self.kind {
            ScreenTransitionKind::Fade => {
                let t = self.tween.progress(now);
                1.0 - (2.0 * t - 1.0).abs()
            }
            _ => 0.0,
        }
    }

    pub fn previous_offset(&self, now: f64, width: f32) -> f32 {
        let travelled = self.tween.progress(now) * width;
        match self.kind {
            ScreenTransitionKind::SlideLeft => -travelled,
            ScreenTransitionKind::SlideRight => travelled,
            _ => 0.0,
        }
    }
}
