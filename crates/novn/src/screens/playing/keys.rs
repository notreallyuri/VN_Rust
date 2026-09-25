use raylib::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PlayingKeys {
    pub hide: Vec<KeyboardKey>,
    pub skip_toggle: Vec<KeyboardKey>,
    pub skip_hold: Vec<KeyboardKey>,
    pub auto: Vec<KeyboardKey>,
    pub log: Vec<KeyboardKey>,
    pub screenshot: Vec<KeyboardKey>,
    pub fullscreen: Vec<KeyboardKey>,
    pub middle_click_hides: bool,
    pub right_click_pauses: bool,
}

impl Default for PlayingKeys {
    fn default() -> Self {
        Self {
            hide: vec![KeyboardKey::KEY_H],
            skip_toggle: vec![KeyboardKey::KEY_TAB],
            skip_hold: vec![
                KeyboardKey::KEY_LEFT_CONTROL,
                KeyboardKey::KEY_RIGHT_CONTROL,
            ],
            auto: vec![KeyboardKey::KEY_A],
            log: vec![KeyboardKey::KEY_L],
            screenshot: vec![KeyboardKey::KEY_S],
            fullscreen: vec![KeyboardKey::KEY_F],
            middle_click_hides: true,
            right_click_pauses: true,
        }
    }
}

impl PlayingKeys {
    pub fn hide(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.hide = keys.into_iter().collect();
        self
    }

    pub fn skip_toggle(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.skip_toggle = keys.into_iter().collect();
        self
    }

    pub fn skip_hold(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.skip_hold = keys.into_iter().collect();
        self
    }

    pub fn auto(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.auto = keys.into_iter().collect();
        self
    }

    pub fn log(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.log = keys.into_iter().collect();
        self
    }

    pub fn screenshot(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.screenshot = keys.into_iter().collect();
        self
    }

    pub fn fullscreen(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.fullscreen = keys.into_iter().collect();
        self
    }

    pub fn middle_click_hides(mut self, enabled: bool) -> Self {
        self.middle_click_hides = enabled;
        self
    }

    pub fn right_click_pauses(mut self, enabled: bool) -> Self {
        self.right_click_pauses = enabled;
        self
    }
}
