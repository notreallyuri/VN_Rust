use vn_engine::frame::stage::{Stage, progress, stage_layout};
use vn_engine::screens::playing::PlayingConfig;
use vn_engine::script::{StoryVm, TransitionKind};

const STORY: &str = r#"
scene a:
  background hall
  show mary tired at left
  "one"
  show mary happy with dissolve
  show hugo neutral with slide_left
  "two"
  show mary happy at right with dissolve
  "three"
  remove hugo with slide_right
  background street with fade 2
  "four"
  clear with dissolve
  "five"
  show mary happy
  background none
  "six"
"#;

fn play_to(vm: &mut StoryVm, stage: &mut Stage, config: &PlayingConfig, now: f64) {
    loop {
        let event = vm.advance();
        if event.is_blocking() {
            return;
        }
        stage.apply(&event, vm, config, now);
    }
}

#[test]
fn events_start_the_right_animations() {
    let config = PlayingConfig::default();
    let mut vm = StoryVm::from_source(STORY);
    let mut stage = Stage::default();

    stage.sync(&vm, &config);
    play_to(&mut vm, &mut stage, &config, 0.0);
    assert!(!stage.is_animating(), "no `with`, no animation");

    play_to(&mut vm, &mut stage, &config, 10.0);
    let mary = stage.character("mary").expect("an expression change");
    assert_eq!(mary.from_image.as_deref(), Some("tired"));
    assert_eq!(mary.from_x, None);
    assert_eq!(stage.texture_paths(), ["characters/mary/tired.png"]);
    let hugo = stage.character("hugo").expect("an entrance");
    assert_eq!(hugo.transition.kind, TransitionKind::SlideLeft);
    assert!(hugo.from_x.unwrap() > 1.0, "slides in from the right edge");

    stage.expire(10.4);
    assert!(stage.character("mary").is_some());
    stage.expire(10.55);
    assert!(stage.character("mary").is_none(), "dissolve lasts 0.5 s");

    play_to(&mut vm, &mut stage, &config, 20.0);
    let moved = stage.character("mary").expect("a move");
    assert_eq!(moved.from_image, None);
    assert_eq!(
        moved.from_x,
        Some(config.position_x(vn_engine::script::Position::Left))
    );

    play_to(&mut vm, &mut stage, &config, 30.0);
    let leaving = stage.character("hugo").expect("an exit");
    assert!(leaving.leaving.is_some());
    let backdrop = stage.background_anim().expect("a background change");
    assert_eq!(backdrop.from.as_deref(), Some("hall"));
    assert_eq!(progress(backdrop.start, &backdrop.transition, 31.0), 0.5);
    let mut paths = stage.texture_paths();
    paths.sort();
    assert_eq!(
        paths,
        ["backgrounds/hall.png", "characters/hugo/neutral.png"],
        "what's fading out stays loaded"
    );

    play_to(&mut vm, &mut stage, &config, 40.0);
    assert!(
        stage.character("mary").unwrap().leaving.is_some(),
        "clear fades everyone out"
    );

    play_to(&mut vm, &mut stage, &config, 41.0);
    assert!(
        stage.character("mary").is_none(),
        "showing without `with` cuts"
    );
    assert!(stage.background_anim().is_none());
}

#[test]
fn finishing_and_resetting_drop_every_animation() {
    let config = PlayingConfig::default();
    let mut vm = StoryVm::from_source(STORY);
    let mut stage = Stage::default();
    stage.sync(&vm, &config);
    play_to(&mut vm, &mut stage, &config, 0.0);
    play_to(&mut vm, &mut stage, &config, 1.0);
    assert!(stage.is_animating());
    stage.finish();
    assert!(!stage.is_animating());

    play_to(&mut vm, &mut stage, &config, 2.0);
    stage.reset(&vm, &config);
    assert!(!stage.is_animating());
    assert!(stage.is_synced());
}

#[test]
fn the_layout_spreads_unplaced_characters() {
    let config = PlayingConfig::default();
    let mut vm = StoryVm::from_source(
        "scene a:\n  show hugo neutral\n  show mary happy\n  show adelaide calm at right\n  \"x\"\n",
    );
    vm.advance_until_blocking();
    let layout = stage_layout(&vm, &config);
    assert_eq!(
        layout["hugo"].x,
        1.0 / 3.0,
        "unplaced characters go in id order"
    );
    assert_eq!(layout["mary"].x, 2.0 / 3.0);
    assert_eq!(layout["adelaide"].x, 0.7);
}
