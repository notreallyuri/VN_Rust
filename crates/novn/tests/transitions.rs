use novn::frame::screen_transition::{
    ScreenTransition, ScreenTransitionConfig, ScreenTransitionKind,
};
use novn::ui::ease::{Easing, Tween};

#[test]
fn easing_stays_inside_its_range() {
    for easing in [Easing::Linear, Easing::Smooth, Easing::In, Easing::Out] {
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(1.0), 1.0);
        assert_eq!(easing.apply(-1.0), 0.0);
        assert_eq!(easing.apply(2.0), 1.0);
        let middle = easing.apply(0.5);
        assert!(
            (0.0..=1.0).contains(&middle),
            "{:?} gave {}",
            easing,
            middle
        );
    }
}

#[test]
fn smooth_easing_matches_smoothstep() {
    assert_eq!(Easing::Smooth.apply(0.5), 0.5);
    assert!(Easing::Smooth.apply(0.25) < 0.25);
    assert!(Easing::Smooth.apply(0.75) > 0.75);
}

#[test]
fn a_tween_runs_from_zero_to_one_over_its_seconds() {
    let tween = Tween::linear(10.0, 2.0);
    assert_eq!(tween.elapsed(10.0), 0.0);
    assert_eq!(tween.elapsed(11.0), 0.5);
    assert_eq!(tween.elapsed(12.0), 1.0);
    assert_eq!(tween.elapsed(99.0), 1.0);
    assert!(!tween.finished(11.0));
    assert!(tween.finished(12.0));
    assert_eq!(tween.value(11.0, 100.0, 200.0), 150.0);
}

#[test]
fn a_tween_with_no_time_is_done_at_once() {
    let tween = Tween::linear(10.0, 0.0);
    assert_eq!(tween.elapsed(10.0), 1.0);
    assert!(tween.finished(10.0));
}

#[test]
fn no_transition_is_started_when_it_is_turned_off() {
    assert!(ScreenTransition::start(ScreenTransitionConfig::none(), 0.0).is_none());
    assert!(
        ScreenTransition::start(
            ScreenTransitionConfig::new(ScreenTransitionKind::Crossfade, 0.0),
            0.0
        )
        .is_none()
    );
}

#[test]
fn a_crossfade_fades_the_previous_screen_out() {
    let config = ScreenTransitionConfig::new(ScreenTransitionKind::Crossfade, 1.0);
    let transition = ScreenTransition::start(config, 0.0).unwrap();
    assert_eq!(transition.previous_alpha(0.0), 1.0);
    assert_eq!(transition.previous_alpha(0.5), 0.5);
    assert_eq!(transition.previous_alpha(1.0), 0.0);
    assert_eq!(transition.cover_alpha(0.5), 0.0);
    assert!(transition.finished(1.0));
}

#[test]
fn a_fade_goes_through_black_and_swaps_at_the_midpoint() {
    let config = ScreenTransitionConfig::new(ScreenTransitionKind::Fade, 1.0);
    let transition = ScreenTransition::start(config, 0.0).unwrap();
    assert_eq!(transition.previous_alpha(0.25), 1.0);
    assert_eq!(transition.previous_alpha(0.75), 0.0);
    assert_eq!(transition.cover_alpha(0.0), 0.0);
    assert_eq!(transition.cover_alpha(0.5), 1.0);
    assert_eq!(transition.cover_alpha(1.0), 0.0);
}

#[test]
fn a_slide_moves_the_previous_screen_off_the_side_it_names() {
    let left = ScreenTransition::start(
        ScreenTransitionConfig::new(ScreenTransitionKind::SlideLeft, 1.0),
        0.0,
    )
    .unwrap();
    assert_eq!(left.previous_offset(0.0, 800.0), 0.0);
    assert_eq!(left.previous_offset(1.0, 800.0), -800.0);
    assert_eq!(left.previous_alpha(0.5), 1.0);

    let right = ScreenTransition::start(
        ScreenTransitionConfig::new(ScreenTransitionKind::SlideRight, 1.0),
        0.0,
    )
    .unwrap();
    assert_eq!(right.previous_offset(1.0, 800.0), 800.0);
}
