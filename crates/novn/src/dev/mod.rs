pub mod capture;
pub mod inspector;
pub mod scene_jump;

use raylib::prelude::KeyboardKey;

pub const CAPTURE_KEY: KeyboardKey = KeyboardKey::KEY_F12;
pub const INSPECTOR_KEY: KeyboardKey = KeyboardKey::KEY_F3;
pub const SCENE_JUMP_KEY: KeyboardKey = KeyboardKey::KEY_F4;
