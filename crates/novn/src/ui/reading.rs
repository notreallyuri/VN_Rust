use std::cell::Cell;

use crate::ui::TextStyle;

pub const SIZES: [u32; 5] = [90, 100, 115, 130, 150];

thread_local! {
    static SCALE: Cell<f32> = const { Cell::new(1.0) };
}

pub fn scale() -> f32 {
    SCALE.with(Cell::get)
}

pub fn set_percent(percent: u32) {
    let percent = percent.clamp(SIZES[0], SIZES[SIZES.len() - 1]);
    SCALE.with(|scale| scale.set(percent as f32 / 100.0));
}

pub fn text(style: &TextStyle) -> TextStyle {
    let mut style = style.clone();
    style.size *= scale();
    style
}

pub fn nearest(percent: u32) -> usize {
    SIZES
        .iter()
        .enumerate()
        .min_by_key(|(_, size)| size.abs_diff(percent))
        .map_or(1, |(index, _)| index)
}
