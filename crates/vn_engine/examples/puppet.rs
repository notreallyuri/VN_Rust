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

    let shot = Cell::new(std::env::var_os("VN_SHOT").is_none());
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
        .screen(ScreenState::Custom("portrait".into()), || PortraitScreen {
            frames: 0,
        })
        .command_as("portrait", |_, (): ()| {
            Some(ScreenState::Custom("portrait".into()))
        })
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

/// A screen of the engine's own, showing a character its backend draws: it asks for the
/// appearance each frame the way the playing screen does, then draws it where it likes.
#[cfg(feature = "character-visuals")]
struct PortraitScreen {
    frames: u32,
}

#[cfg(feature = "character-visuals")]
impl vn_engine::prelude::Screen for PortraitScreen {
    fn update(
        &mut self,
        mut ctx: vn_engine::prelude::GameContext,
    ) -> Option<vn_engine::prelude::ScreenState> {
        use vn_engine::raylib::prelude::KeyboardKey;
        ctx.show_visual("mary", "happy");
        self.frames += 1;
        if ctx.rl.is_key_pressed(KeyboardKey::KEY_S)
            || (self.frames == 40 && std::env::var_os("VN_SHOT").is_some())
        {
            ctx.screenshot();
        }
        let leave = ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
            || ctx.rl.is_key_pressed(KeyboardKey::KEY_SPACE);
        leave.then_some(vn_engine::prelude::ScreenState::Playing)
    }

    fn draw(
        &self,
        d: &mut vn_engine::raylib::prelude::RaylibDrawHandle,
        ctx: &vn_engine::prelude::DrawContext,
    ) {
        use vn_engine::raylib::prelude::*;
        let screen = vn_engine::ui::screen_size(d);
        d.clear_background(Color::new(18, 16, 24, 255));
        let drawn = ctx
            .resources
            .visuals
            .draw("mary", "happy", d, screen, 0.5, Some(0.95), 1.0);
        vn_engine::ui::draw_text_centered(
            d,
            ctx.fonts(),
            match drawn {
                true => "A screen of the game's own, drawing the puppet. Space to go back.",
                false => "The backend drew nothing here.",
            },
            Vector2::new(screen.x * 0.5, screen.y * 0.08),
            &vn_engine::prelude::TextStyle::new(
                vn_engine::prelude::FontRole::Default,
                22.0,
                Color::WHITE,
            ),
        );
    }
}

#[cfg(not(feature = "character-visuals"))]
fn main() {
    eprintln!("Enable character-visuals to run this example");
}
