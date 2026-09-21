use raylib::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputDevice {
    #[default]
    Mouse,
    Keyboard,
    Gamepad,
}

impl InputDevice {
    pub fn name(self) -> &'static str {
        match self {
            InputDevice::Mouse => "mouse",
            InputDevice::Keyboard => "keyboard",
            InputDevice::Gamepad => "gamepad",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NavInput {
    pub device: InputDevice,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub next: bool,
    pub previous: bool,
    pub accept: bool,
    pub back: bool,
    pub pause: bool,
    pub alt: bool,
    pub page_back: bool,
    pub page_forward: bool,
    pub hide: bool,
    pub log: bool,
    pub skip_held: bool,
    pub pointer: bool,
}

impl NavInput {
    pub fn direction(&self) -> Option<Vector2> {
        match (self.up, self.down, self.left, self.right) {
            (true, false, _, _) => Some(Vector2::new(0.0, -1.0)),
            (false, true, _, _) => Some(Vector2::new(0.0, 1.0)),
            (_, _, true, false) => Some(Vector2::new(-1.0, 0.0)),
            (_, _, false, true) => Some(Vector2::new(1.0, 0.0)),
            _ => None,
        }
    }

    pub fn horizontal(&self) -> i32 {
        i32::from(self.right) - i32::from(self.left)
    }

    pub fn vertical(&self) -> i32 {
        i32::from(self.down) - i32::from(self.up)
    }

    pub fn any_key(&self) -> bool {
        self.up
            || self.down
            || self.left
            || self.right
            || self.next
            || self.previous
            || self.accept
            || self.back
            || self.pause
            || self.alt
            || self.page_back
            || self.page_forward
            || self.hide
            || self.log
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavigationConfig {
    pub enabled: bool,
    pub gamepad: bool,
    pub up_keys: Vec<KeyboardKey>,
    pub down_keys: Vec<KeyboardKey>,
    pub left_keys: Vec<KeyboardKey>,
    pub right_keys: Vec<KeyboardKey>,
    pub accept_keys: Vec<KeyboardKey>,
    pub back_keys: Vec<KeyboardKey>,
    pub alt_keys: Vec<KeyboardKey>,
    pub stick_deadzone: f32,
    pub repeat_delay: f64,
    pub repeat_interval: f64,
}

impl Default for NavigationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            gamepad: true,
            up_keys: vec![KeyboardKey::KEY_UP],
            down_keys: vec![KeyboardKey::KEY_DOWN],
            left_keys: vec![KeyboardKey::KEY_LEFT],
            right_keys: vec![KeyboardKey::KEY_RIGHT],
            accept_keys: vec![
                KeyboardKey::KEY_ENTER,
                KeyboardKey::KEY_KP_ENTER,
                KeyboardKey::KEY_SPACE,
            ],
            back_keys: Vec::new(),
            alt_keys: vec![KeyboardKey::KEY_DELETE],
            stick_deadzone: 0.5,
            repeat_delay: 0.4,
            repeat_interval: 0.12,
        }
    }
}

impl NavigationConfig {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn gamepad(mut self, enabled: bool) -> Self {
        self.gamepad = enabled;
        self
    }

    pub fn up_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.up_keys = keys.into_iter().collect();
        self
    }

    pub fn down_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.down_keys = keys.into_iter().collect();
        self
    }

    pub fn left_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.left_keys = keys.into_iter().collect();
        self
    }

    pub fn right_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.right_keys = keys.into_iter().collect();
        self
    }

    pub fn accept_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.accept_keys = keys.into_iter().collect();
        self
    }

    pub fn back_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.back_keys = keys.into_iter().collect();
        self
    }

    pub fn alt_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.alt_keys = keys.into_iter().collect();
        self
    }

    pub fn stick_deadzone(mut self, deadzone: f32) -> Self {
        self.stick_deadzone = deadzone.clamp(0.05, 0.95);
        self
    }

    pub fn repeat(mut self, delay: f64, interval: f64) -> Self {
        self.repeat_delay = delay.max(0.0);
        self.repeat_interval = interval.max(0.01);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Repeater {
    held_since: Option<f64>,
    last_fire: f64,
}

impl Repeater {
    pub fn update(&mut self, held: bool, now: f64, delay: f64, interval: f64) -> bool {
        if !held {
            self.held_since = None;
            return false;
        }
        match self.held_since {
            None => {
                self.held_since = Some(now);
                self.last_fire = now;
                true
            }
            Some(since) if now - since >= delay && now - self.last_fire >= interval => {
                self.last_fire = now;
                true
            }
            Some(_) => false,
        }
    }
}

#[derive(Debug)]
pub struct Navigation {
    pub config: NavigationConfig,
    repeaters: [Repeater; 4],
    pointer: bool,
    device: InputDevice,
}

impl Default for Navigation {
    fn default() -> Self {
        Self::new(NavigationConfig::default())
    }
}

const GAMEPADS: i32 = 4;

impl Navigation {
    pub fn new(config: NavigationConfig) -> Self {
        Self {
            config,
            repeaters: [Repeater::default(); 4],
            pointer: true,
            device: InputDevice::Mouse,
        }
    }

    pub fn device(&self) -> InputDevice {
        self.device
    }

    pub fn read(&mut self, rl: &RaylibHandle) -> NavInput {
        if !self.config.enabled {
            self.device = InputDevice::Mouse;
            return NavInput {
                pointer: true,
                device: InputDevice::Mouse,
                ..NavInput::default()
            };
        }

        let config = &self.config;
        let keys = |list: &[KeyboardKey]| {
            list.iter()
                .any(|&key| rl.is_key_pressed(key) || rl.is_key_pressed_repeat(key))
        };
        let once = |list: &[KeyboardKey]| list.iter().any(|&key| rl.is_key_pressed(key));
        let shift = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);
        let tab = keys(&[KeyboardKey::KEY_TAB]);

        let mut input = NavInput {
            up: keys(&config.up_keys),
            down: keys(&config.down_keys),
            left: keys(&config.left_keys),
            right: keys(&config.right_keys),
            next: tab && !shift,
            previous: tab && shift,
            accept: once(&config.accept_keys),
            back: once(&config.back_keys),
            alt: once(&config.alt_keys),
            ..NavInput::default()
        };

        let typed = input.any_key();
        let mut padded = false;

        if config.gamepad
            && let Some(pad) = (0..GAMEPADS).find(|&pad| rl.is_gamepad_available(pad))
        {
            use GamepadButton::*;
            let pressed = |button| rl.is_gamepad_button_pressed(pad, button);
            let down = |button| rl.is_gamepad_button_down(pad, button);
            let x = rl.get_gamepad_axis_movement(pad, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
            let y = rl.get_gamepad_axis_movement(pad, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
            let dead = config.stick_deadzone;
            let held = [
                down(GAMEPAD_BUTTON_LEFT_FACE_UP) || y < -dead,
                down(GAMEPAD_BUTTON_LEFT_FACE_DOWN) || y > dead,
                down(GAMEPAD_BUTTON_LEFT_FACE_LEFT) || x < -dead,
                down(GAMEPAD_BUTTON_LEFT_FACE_RIGHT) || x > dead,
            ];

            let now = rl.get_time();
            let (delay, interval) = (config.repeat_delay, config.repeat_interval);
            let fired: Vec<bool> = self
                .repeaters
                .iter_mut()
                .zip(held)
                .map(|(repeater, held)| repeater.update(held, now, delay, interval))
                .collect();

            input.up |= fired[0];
            input.down |= fired[1];
            input.left |= fired[2];
            input.right |= fired[3];
            input.accept |= pressed(GAMEPAD_BUTTON_RIGHT_FACE_DOWN);
            input.back |= pressed(GAMEPAD_BUTTON_RIGHT_FACE_RIGHT);
            input.alt |= pressed(GAMEPAD_BUTTON_RIGHT_FACE_LEFT);
            input.pause |= pressed(GAMEPAD_BUTTON_MIDDLE_RIGHT);
            input.page_back |= pressed(GAMEPAD_BUTTON_LEFT_TRIGGER_1);
            input.page_forward |= pressed(GAMEPAD_BUTTON_RIGHT_TRIGGER_1);
            input.hide |= pressed(GAMEPAD_BUTTON_MIDDLE_LEFT);
            input.log |= pressed(GAMEPAD_BUTTON_RIGHT_FACE_UP);
            input.skip_held = down(GAMEPAD_BUTTON_RIGHT_TRIGGER_2)
                || rl.get_gamepad_axis_movement(pad, GamepadAxis::GAMEPAD_AXIS_RIGHT_TRIGGER) > 0.5;
            padded = input.any_key() && !typed || input.skip_held;
        }

        let mouse_used = rl.get_mouse_delta() != Vector2::zero()
            || rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            || rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT)
            || rl.get_mouse_wheel_move() != 0.0;
        if input.any_key() {
            self.pointer = false;
        } else if mouse_used {
            self.pointer = true;
        }

        if padded {
            self.device = InputDevice::Gamepad;
        } else if typed {
            self.device = InputDevice::Keyboard;
        } else if mouse_used {
            self.device = InputDevice::Mouse;
        }

        input.pointer = self.pointer;
        input.device = self.device;
        input
    }
}

struct Span {
    start: f32,
    end: f32,
}

impl Span {
    fn center(&self) -> f32 {
        (self.start + self.end) / 2.0
    }

    fn gap(&self, other: &Span) -> f32 {
        (other.start - self.end)
            .max(self.start - other.end)
            .max(0.0)
    }
}

fn spans(rect: &Rectangle, direction: Vector2) -> (Span, Span) {
    let x = Span {
        start: rect.x,
        end: rect.x + rect.width,
    };
    let y = Span {
        start: rect.y,
        end: rect.y + rect.height,
    };
    if direction.x != 0.0 { (x, y) } else { (y, x) }
}

pub fn navigate(
    rects: &[Rectangle],
    enabled: &[bool],
    from: usize,
    direction: Vector2,
    wrap: bool,
) -> Option<usize> {
    let (from_along, from_across) = spans(rects.get(from)?, direction);
    let sign = if direction.x + direction.y > 0.0 {
        1.0
    } else {
        -1.0
    };
    let usable = |j: usize| j != from && enabled.get(j).copied().unwrap_or(true);

    let measured: Vec<(usize, f32, f32, f32)> = rects
        .iter()
        .enumerate()
        .filter(|&(j, _)| usable(j))
        .map(|(j, rect)| {
            let (along, across) = spans(rect, direction);
            let ahead = if sign > 0.0 {
                along.start - from_along.end
            } else {
                from_along.start - along.end
            };
            let tolerance = 0.5 * (along.end - along.start).min(from_along.end - from_along.start);
            let ahead = if ahead > -tolerance && (along.center() - from_along.center()) * sign > 0.0
            {
                ahead.max(0.0)
            } else {
                f32::NAN
            };
            let side = from_across.gap(&across);
            let offset = (across.center() - from_across.center()).abs();
            (j, ahead, side, offset)
        })
        .collect();

    let score = |ahead: f32, side: f32| ahead + side * 3.0;
    let best = measured
        .iter()
        .filter(|(_, ahead, _, _)| !ahead.is_nan())
        .min_by(|a, b| {
            score(a.1, a.2)
                .total_cmp(&score(b.1, b.2))
                .then(a.3.total_cmp(&b.3))
        })
        .map(|(j, _, _, _)| *j);

    best.or_else(|| {
        if !wrap {
            return None;
        }
        rects
            .iter()
            .enumerate()
            .filter(|&(j, _)| usable(j))
            .filter_map(|(j, rect)| {
                let (along, across) = spans(rect, direction);
                let behind = (from_along.center() - along.center()) * sign;
                (behind > 1.0 && from_across.gap(&across) == 0.0).then(|| {
                    let offset = (across.center() - from_across.center()).abs();
                    (j, behind, offset)
                })
            })
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.2.total_cmp(&a.2)))
            .map(|(j, _, _)| j)
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Focus {
    index: Option<usize>,
}

impl Focus {
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn set(&mut self, index: Option<usize>) {
        self.index = index;
    }

    pub fn is(&self, index: usize) -> bool {
        self.index == Some(index)
    }

    pub fn update(
        &mut self,
        nav: &NavInput,
        rects: &[Rectangle],
        enabled: &[bool],
        hovered: Option<usize>,
    ) -> Option<usize> {
        let usable = |i: usize| i < rects.len() && enabled.get(i).copied().unwrap_or(true);

        if self.index.is_some_and(|i| !usable(i)) {
            self.index = None;
        }
        if nav.pointer
            && let Some(i) = hovered.filter(|&i| usable(i))
        {
            self.index = Some(i);
        }

        let first = (0..rects.len()).find(|&i| usable(i));
        let moved = nav.direction().is_some() || nav.next || nav.previous;
        match self.index {
            None if moved || nav.accept || !nav.pointer => {
                self.index = first;
                return None;
            }
            None => return None,
            Some(from) => {
                if let Some(direction) = nav.direction() {
                    if let Some(to) = navigate(rects, enabled, from, direction, true) {
                        self.index = Some(to);
                    }
                } else if nav.next || nav.previous {
                    let len = rects.len();
                    let step = |i: usize| {
                        if nav.next {
                            (i + 1) % len
                        } else {
                            (i + len - 1) % len
                        }
                    };
                    let mut i = step(from);
                    while i != from && !usable(i) {
                        i = step(i);
                    }
                    self.index = Some(i);
                }
            }
        }

        if nav.accept { self.index } else { None }
    }
}
