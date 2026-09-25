#[cfg(feature = "character-visuals")]
fn main() -> Result<(), novn::app::AppError> {
    use std::cell::Cell;
    use std::rc::Rc;

    use novn::game::puppet::{BLINK, BREATH, SWAY};
    use novn::prelude::*;

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
    let syncing = Rc::new(Cell::new(false));
    let lips = Rc::clone(&syncing);
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
        .audio(|audio| audio.enabled(true))
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
        .command_as("lipsync", move |_, (): ()| {
            lips.set(true);
            println!("The mouth is following the voice line now");
            None
        })
        .command_as("quiet", move |ctx, (): ()| {
            stop.set(false);
            ctx.visual_parameter("mary", "mouth", None);
            println!("Released it back to the pose and the idle clock");
            None
        })
        .on_frame(move |ctx, seconds| {
            if syncing.get() {
                let level = ctx.voice_level();
                ctx.visual_parameter("mary", "mouth", Some(level));
                if level > 0.0 {
                    print!("\rvoice {level:.2} ");
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                }
            }
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

#[cfg(feature = "character-visuals")]
struct PortraitScreen {
    frames: u32,
}

#[cfg(feature = "character-visuals")]
impl novn::prelude::Screen for PortraitScreen {
    fn update(
        &mut self,
        mut ctx: novn::prelude::GameContext,
    ) -> Option<novn::prelude::ScreenState> {
        use novn::raylib::prelude::KeyboardKey;
        ctx.show_visual("mary", "happy");
        self.frames += 1;
        if ctx.rl.is_key_pressed(KeyboardKey::KEY_S)
            || (self.frames == 40 && std::env::var_os("VN_SHOT").is_some())
        {
            ctx.screenshot();
        }
        let leave = ctx.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
            || ctx.rl.is_key_pressed(KeyboardKey::KEY_SPACE);
        leave.then_some(novn::prelude::ScreenState::Playing)
    }

    fn draw(
        &self,
        d: &mut novn::raylib::prelude::RaylibDrawHandle,
        ctx: &novn::prelude::DrawContext,
    ) {
        use novn::raylib::prelude::*;
        let screen = novn::ui::screen_size(d);
        d.clear_background(Color::new(18, 16, 24, 255));
        let drawn = ctx
            .resources
            .visuals
            .draw("mary", "happy", d, screen, 0.5, Some(0.95), 1.0);
        novn::ui::draw_text_centered(
            d,
            ctx.fonts(),
            match drawn {
                true => "A screen of the game's own, drawing the puppet. Space to go back.",
                false => "The backend drew nothing here.",
            },
            Vector2::new(screen.x * 0.5, screen.y * 0.08),
            &novn::prelude::TextStyle::new(novn::prelude::FontRole::Default, 22.0, Color::WHITE),
        );
    }
}

#[cfg(not(feature = "character-visuals"))]
fn main() {
    eprintln!("Enable character-visuals to run this example");
}
