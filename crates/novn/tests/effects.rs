use novn::frame::effects::{ScreenEffects, ScreenEffectsConfig};

fn effects() -> ScreenEffects {
    ScreenEffects::new(ScreenEffectsConfig::default())
}

#[test]
fn nothing_moves_without_an_effect() {
    let effects = effects();
    assert_eq!(effects.offset(1.0).x, 0.0);
    assert_eq!(effects.offset(1.0).y, 0.0);
    assert_eq!(effects.flash_alpha(1.0), 0.0);
    assert!(!effects.active(1.0));
}

#[test]
fn a_shake_dies_down_and_ends_where_it_started() {
    let mut effects = effects();
    effects.shake(0.0, 0.5);

    assert!(effects.active(0.1));
    let early = effects.offset(0.05).x.abs() + effects.offset(0.05).y.abs();
    let late = effects.offset(0.45).x.abs() + effects.offset(0.45).y.abs();
    assert!(early > late, "shake should fade: {} then {}", early, late);

    assert_eq!(effects.offset(0.5).x, 0.0);
    assert_eq!(effects.offset(0.5).y, 0.0);
    assert!(!effects.active(0.5));
}

#[test]
fn a_shake_stays_within_its_strength() {
    let mut effects = effects();
    effects.shake(0.0, 1.0);
    let limit = ScreenEffectsConfig::default().shake_strength;

    for step in 0..100 {
        let offset = effects.offset(step as f64 / 100.0);
        assert!(offset.x.abs() <= limit, "x drifted to {}", offset.x);
        assert!(offset.y.abs() <= limit, "y drifted to {}", offset.y);
    }
}

#[test]
fn a_flash_starts_bright_and_fades_out() {
    let mut effects = effects();
    effects.flash(0.0, 1.0);

    assert_eq!(effects.flash_alpha(0.0), 1.0);
    assert!(effects.flash_alpha(0.5) < 1.0);
    assert!(effects.flash_alpha(0.5) > 0.0);
    assert_eq!(effects.flash_alpha(1.0), 0.0);
    assert!(!effects.active(1.0));
}

#[test]
fn an_effect_with_no_time_never_starts() {
    let mut effects = effects();
    effects.shake(0.0, 0.0);
    effects.flash(0.0, -1.0);
    assert!(!effects.active(0.0));
}

#[test]
fn clearing_stops_everything() {
    let mut effects = effects();
    effects.shake(0.0, 5.0);
    effects.flash(0.0, 5.0);
    assert!(effects.active(1.0));

    effects.clear();
    assert!(!effects.active(1.0));
    assert_eq!(effects.flash_alpha(1.0), 0.0);
}
