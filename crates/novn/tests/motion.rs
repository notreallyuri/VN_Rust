use novn::frame::effects::{ScreenEffects, ScreenEffectsConfig};
use novn::frame::scenery::Motion;
use novn::frame::screen_transition::{
    ScreenTransition, ScreenTransitionConfig, ScreenTransitionKind,
};
use novn::script::TransitionKind;
use novn::ui::motion;

#[test]
fn reduced_motion_turns_slides_into_dissolves_and_leaves_fades_alone() {
    motion::set_reduced(false);
    assert_eq!(
        motion::calm(TransitionKind::SlideLeft),
        TransitionKind::SlideLeft
    );
    motion::set_reduced(true);
    assert_eq!(
        motion::calm(TransitionKind::SlideLeft),
        TransitionKind::Dissolve
    );
    assert_eq!(
        motion::calm(TransitionKind::SlideRight),
        TransitionKind::Dissolve
    );
    assert_eq!(motion::calm(TransitionKind::Fade), TransitionKind::Fade);
    assert_eq!(
        motion::calm(TransitionKind::Dissolve),
        TransitionKind::Dissolve
    );
    motion::set_reduced(false);
}

#[test]
fn reduced_motion_stops_the_shake_and_removes_the_flash() {
    let mut effects = ScreenEffects::new(ScreenEffectsConfig::default());
    effects.shake(0.0, 1.0);
    effects.flash(0.0, 1.0);
    motion::set_reduced(false);
    assert!(effects.offset(0.3).length() > 0.0 || effects.offset(0.1).length() > 0.0);
    assert!(effects.flash_alpha(0.1) > 0.0);
    motion::set_reduced(true);
    assert_eq!(effects.offset(0.1).length(), 0.0);
    assert_eq!(
        effects.flash_alpha(0.1),
        0.0,
        "a flash is a photosensitivity risk, not just motion"
    );
    motion::set_reduced(false);
}

#[test]
fn reduced_motion_holds_the_background_still() {
    let moving = Motion::new(1.2, 10.0).pan(0.5, 0.0);
    motion::set_reduced(false);
    assert_ne!(moving.at(2.0), moving.at(5.0));
    motion::set_reduced(true);
    assert_eq!(
        moving.at(2.0),
        moving.at(5.0),
        "the same frame whatever the time"
    );
    assert_eq!(moving.at(2.0).1.x, 0.0, "held in the middle of its pan");
    motion::set_reduced(false);
}

#[test]
fn reduced_motion_crossfades_between_screens_instead_of_sliding() {
    let slide = ScreenTransition::start(
        ScreenTransitionConfig::new(ScreenTransitionKind::SlideLeft, 0.5),
        0.0,
    )
    .unwrap();
    motion::set_reduced(false);
    assert_eq!(slide.kind(), ScreenTransitionKind::SlideLeft);
    assert!(slide.previous_offset(0.25, 1280.0) != 0.0);
    motion::set_reduced(true);
    assert_eq!(slide.kind(), ScreenTransitionKind::Crossfade);
    assert_eq!(slide.previous_offset(0.25, 1280.0), 0.0);
    motion::set_reduced(false);
}
