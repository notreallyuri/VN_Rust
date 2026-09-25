pub mod capture;
pub mod director;
pub mod inspector;
pub mod picker;
pub mod scene_jump;
pub mod styler;

use raylib::prelude::KeyboardKey;

pub const CAPTURE_KEY: KeyboardKey = KeyboardKey::KEY_F12;
pub const DIRECTOR_KEY: KeyboardKey = KeyboardKey::KEY_F7;
pub const INSPECTOR_KEY: KeyboardKey = KeyboardKey::KEY_F3;
pub const PICKER_KEY: KeyboardKey = KeyboardKey::KEY_F6;
pub const STYLE_EDITOR_KEY: KeyboardKey = KeyboardKey::KEY_F8;
pub const SCENE_JUMP_KEY: KeyboardKey = KeyboardKey::KEY_F4;
