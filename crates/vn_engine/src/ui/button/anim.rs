use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use raylib::prelude::*;

use super::{ButtonStyle, StateAmounts};
use crate::GameContext;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Anim {
    pub(crate) amounts: StateAmounts,
    pub(crate) seen: f64,
}

thread_local! {
    pub(crate) static ANIMATIONS: RefCell<(f64, HashMap<u64, Anim>)> = RefCell::new((0.0, HashMap::new()));
    static PRESSED: RefCell<Option<u64>> = const { RefCell::new(None) };
}

pub(crate) fn button_key(rect: Rectangle, label: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for value in [rect.x, rect.y, rect.width, rect.height] {
        value.to_bits().hash(&mut hasher);
    }
    label.hash(&mut hasher);
    hasher.finish()
}

fn rect_key(rect: Rectangle) -> u64 {
    button_key(rect, "")
}

pub fn step_amount(current: f32, target: f32, dt: f32, seconds: f32) -> f32 {
    if seconds <= 0.0 {
        return target;
    }
    let delta = dt / seconds;
    if current < target {
        (current + delta).min(target)
    } else {
        (current - delta).max(target)
    }
}

pub fn button_hovered(rl: &RaylibHandle, rect: Rectangle, style: &ButtonStyle) -> bool {
    style.contains(rect, crate::frame::viewport::mouse_position(rl))
}

pub fn button_clicked(ctx: &mut GameContext, rect: Rectangle, style: &ButtonStyle) -> bool {
    let rl = &*ctx.rl;
    let mouse = crate::frame::viewport::mouse_position(rl);
    let hovered = style.contains(rect, mouse);
    let key = rect_key(rect);

    let before = mouse - rl.get_mouse_delta();
    let entered = hovered && !style.contains(rect, before);

    let pressed = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
    let released = rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT);

    if pressed && hovered {
        PRESSED.with(|p| *p.borrow_mut() = Some(key));
    }
    let clicked = released && hovered && PRESSED.with(|p| *p.borrow() == Some(key));
    if released && PRESSED.with(|p| *p.borrow() == Some(key)) {
        PRESSED.with(|p| *p.borrow_mut() = None);
    }

    if entered && let Some(id) = &style.hover_sound {
        ctx.play_sound(id);
    }
    if clicked && let Some(id) = &style.click_sound {
        ctx.play_sound(id);
    }
    clicked
}
