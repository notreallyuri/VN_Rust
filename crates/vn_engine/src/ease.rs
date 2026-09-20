#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Easing {
    Linear,
    #[default]
    Smooth,
    In,
    Out,
}

impl Easing {
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::Smooth => t * t * (3.0 - 2.0 * t),
            Easing::In => t * t,
            Easing::Out => t * (2.0 - t),
        }
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[derive(Debug, Clone, Copy)]
pub struct Tween {
    start: f64,
    seconds: f32,
    easing: Easing,
}

impl Tween {
    pub fn new(now: f64, seconds: f32, easing: Easing) -> Self {
        Self {
            start: now,
            seconds,
            easing,
        }
    }

    pub fn linear(now: f64, seconds: f32) -> Self {
        Self::new(now, seconds, Easing::Linear)
    }

    pub fn elapsed(&self, now: f64) -> f32 {
        if self.seconds <= 0.0 {
            return 1.0;
        }
        (((now - self.start) / self.seconds as f64) as f32).clamp(0.0, 1.0)
    }

    pub fn progress(&self, now: f64) -> f32 {
        self.easing.apply(self.elapsed(now))
    }

    pub fn value(&self, now: f64, from: f32, to: f32) -> f32 {
        lerp(from, to, self.progress(now))
    }

    pub fn finished(&self, now: f64) -> bool {
        self.elapsed(now) >= 1.0
    }
}
