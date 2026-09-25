use std::f32::consts::TAU;

use raylib::prelude::*;

use crate::ui::ease::Tween;

#[derive(Debug, Clone, Copy)]
pub struct ScreenEffectsConfig {
    pub shake_strength: f32,
    pub shake_frequency: f32,
    pub flash_color: Color,
}

impl Default for ScreenEffectsConfig {
    fn default() -> Self {
        Self {
            shake_strength: 16.0,
            shake_frequency: 11.0,
            flash_color: Color::WHITE,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Effect {
    tween: Tween,
    seconds: f32,
}

#[derive(Debug, Default)]
pub struct ScreenEffects {
    config: ScreenEffectsConfig,
    shake: Option<Effect>,
    flash: Option<Effect>,
}

impl ScreenEffects {
    pub fn new(config: ScreenEffectsConfig) -> Self {
        Self {
            config,
            shake: None,
            flash: None,
        }
    }

    pub fn config(&self) -> &ScreenEffectsConfig {
        &self.config
    }

    pub fn shake(&mut self, now: f64, seconds: f32) {
        self.shake = start(now, seconds);
    }

    pub fn flash(&mut self, now: f64, seconds: f32) {
        self.flash = start(now, seconds);
    }

    pub fn clear(&mut self) {
        self.shake = None;
        self.flash = None;
    }

    pub fn offset(&self, now: f64) -> Vector2 {
        let Some(shake) = self.shake else {
            return Vector2::zero();
        };
        let t = shake.tween.elapsed(now);
        if t >= 1.0 {
            return Vector2::zero();
        }

        let strength = self.config.shake_strength * (1.0 - t);
        let turns = shake.seconds * self.config.shake_frequency * t * TAU;
        Vector2::new(
            (turns.sin() * strength).round(),
            (turns * 0.7).cos() * strength * 0.5,
        )
    }

    pub fn flash_alpha(&self, now: f64) -> f32 {
        let Some(flash) = self.flash else {
            return 0.0;
        };
        let left = 1.0 - flash.tween.elapsed(now);
        left * left
    }

    pub fn flash_color(&self, now: f64) -> Color {
        self.config.flash_color.alpha(self.flash_alpha(now))
    }

    pub fn active(&self, now: f64) -> bool {
        let running =
            |effect: &Option<Effect>| effect.is_some_and(|effect| !effect.tween.finished(now));
        running(&self.shake) || running(&self.flash)
    }
}

fn start(now: f64, seconds: f32) -> Option<Effect> {
    if seconds <= 0.0 {
        return None;
    }
    Some(Effect {
        tween: Tween::linear(now, seconds),
        seconds,
    })
}
