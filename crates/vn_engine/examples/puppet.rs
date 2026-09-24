#[cfg(feature = "character-visuals")]
fn main() -> Result<(), vn_engine::app::AppError> {
    use std::cell::Cell;
    use std::rc::Rc;

    use vn_engine::game::puppet::{BLINK, BREATH, SWAY};
    use vn_engine::prelude::*;

    let rig = Puppet::new(600.0, 900.0)
        .folder("mary")
        .part(Part::new("body").at(300.0, 900.0).pivot(0.5, 1.0))
        .part(
            Part::new("head")
                .at(300.0, 400.0)
                .pivot(0.5, 1.0)
                .bind(SWAY, Motion::rotate(5.0))
                .bind(BREATH, Motion::offset(0.0, -6.0)),
        )
        .part(
            Part::new("eyes")
                .at(300.0, 250.0)
                .bind(BLINK, Motion::scale(1.0, 0.08))
                .bind(SWAY, Motion::offset(8.0, 0.0)),
        )
        .part(
            Part::new("mouth")
                .at(300.0, 330.0)
                .bind("mouth", Motion::scale(1.0, 2.4)),
        )
        .appearance("neutral", Pose::new())
        .appearance(
            "happy",
            Pose::new()
                .swap("mouth", "mouth_smile")
                .parameter(SWAY, 0.4),
        );

    let shot = Cell::new(!std::env::var_os("VN_SHOT").is_some());
    let talking = Rc::new(Cell::new(false));
    let start = Rc::clone(&talking);
    let stop = Rc::clone(&talking);
    let clock = Cell::new(0.0f32);

    VnApp::new("Puppet")
        .assets(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/puppet"
        ))
        .schema_file(None)
        .saves_dir(std::env::temp_dir().join(format!("vn-puppet-example-{}", std::process::id())))
        .initial_screen(ScreenState::Playing)
        .audio(|audio| audio.enabled(false))
        .on_scene_enter(|ctx, _| {
            ctx.modes.auto = true;
            None
        })
        .character("mary", Character::new("Mary"))
        .character_visual("mary", rig)
        .command_as("talk", move |_, (): ()| {
            start.set(true);
            println!("Pushing mary's mouth from the frame hook");
            None
        })
        .command_as("quiet", move |ctx, (): ()| {
            stop.set(false);
            ctx.visual_parameter("mary", "mouth", None);
            println!("Released it back to the pose and the idle clock");
            None
        })
        .on_frame(move |ctx, seconds| {
            if !talking.get() {
                return;
            }
            clock.set(clock.get() + seconds);
            let open = (clock.get() * 11.0).sin().max(0.0);
            ctx.visual_parameter("mary", "mouth", Some(open));
            if !shot.get() && clock.get() > 0.5 {
                shot.set(true);
                ctx.screenshot();
            }
        })
        .run()
}

#[cfg(not(feature = "character-visuals"))]
fn main() {
    eprintln!("Enable character-visuals to run this example");
}
