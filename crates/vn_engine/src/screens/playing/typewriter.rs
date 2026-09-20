use crate::styled::StyledText;

#[derive(Clone, Debug, PartialEq)]
pub struct Typewriter {
    started: f64,
    chars_per_second: u32,
    total: usize,
    pauses: Vec<(usize, f32)>,
    finished: bool,
}

impl Typewriter {
    pub fn start(text: &str, chars_per_second: u32, now: f64) -> Self {
        let styled = StyledText::parse(text);
        Self {
            started: now,
            chars_per_second,
            total: styled.chars(),
            pauses: styled.pauses(),
            finished: chars_per_second == 0,
        }
    }

    pub fn finished(text: &str) -> Self {
        Self {
            started: 0.0,
            chars_per_second: 0,
            total: StyledText::parse(text).chars(),
            pauses: Vec::new(),
            finished: true,
        }
    }

    pub fn visible(&self, now: f64) -> usize {
        if self.finished || self.chars_per_second == 0 {
            return self.total;
        }

        let speed = self.chars_per_second as f64;
        let mut left = (now - self.started).max(0.0);
        let mut shown = 0;

        for (at, seconds) in &self.pauses {
            let typing = (at - shown) as f64 / speed;
            if left < typing {
                return shown + (left * speed) as usize;
            }
            left -= typing;
            shown = *at;
            if left < *seconds as f64 {
                return shown;
            }
            left -= *seconds as f64;
        }

        (shown + (left * speed) as usize).min(self.total)
    }

    pub fn is_done(&self, now: f64) -> bool {
        self.visible(now) >= self.total
    }

    pub fn finish(&mut self) {
        self.finished = true;
    }
}
