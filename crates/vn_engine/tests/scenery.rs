use vn_engine::action::Action;
use vn_engine::frame::scenery::{Letterbox, Motion, Scenery};
use vn_engine::raylib::prelude::*;
use vn_engine::screens::main_menu::MainMenuConfig;
use vn_engine::screens::playing::{HudButton, PlayingConfig};
use vn_engine::ui::layout::Anchor;

#[test]
fn motion_breathes_between_rest_and_full_zoom() {
    let motion = Motion::new(1.1, 40.0).pan(0.5, -0.5);
    let (zoom, pan) = motion.at(0.0);
    assert!((zoom - 1.0).abs() < 1e-5);
    assert_eq!(pan, Vector2::new(-0.5, 0.5));

    let (zoom, pan) = motion.at(20.0);
    assert!((zoom - 1.1).abs() < 1e-5);
    assert!((pan.x - 0.5).abs() < 1e-5 && (pan.y + 0.5).abs() < 1e-5);

    let (zoom, _) = motion.at(40.0);
    assert!((zoom - 1.0).abs() < 1e-5);
    assert_eq!(Motion::new(0.5, 0.0).zoom, 1.0);
}

#[test]
fn letterbox_slides_in_and_frames_the_screen() {
    let bars = Letterbox::default().height(80.0).slide_in(1.0);
    assert_eq!(bars.height_at(0.0), 0.0);
    assert!(bars.height_at(0.5) > 40.0 && bars.height_at(0.5) < 80.0);
    assert_eq!(bars.height_at(2.0), 80.0);
    assert_eq!(Letterbox::default().height(80.0).height_at(0.0), 80.0);

    let (top, bottom) = bars.bars(Vector2::new(1280.0, 720.0), 5.0);
    assert_eq!((top.y, top.height, top.width), (0.0, 80.0, 1280.0));
    assert_eq!((bottom.y, bottom.height), (640.0, 80.0));
}

#[test]
fn scenery_builder() {
    let scenery = Scenery::default()
        .pan(0.3, 0.2)
        .motion(1.05, 30.0)
        .vignette(Color::BLACK, 0.9)
        .letterbox(|l| l.height(60.0));
    let motion = scenery.motion.unwrap();
    assert_eq!((motion.zoom, motion.period), (1.05, 30.0));
    assert_eq!(motion.pan, Vector2::new(0.3, 0.2));
    assert_eq!(scenery.vignette.unwrap().size, 0.5);
    assert_eq!(
        scenery.bottom_bar(Vector2::new(1280.0, 720.0), 0.0),
        Some(Rectangle::new(0.0, 660.0, 1280.0, 60.0))
    );
    assert_eq!(
        Scenery::default().bottom_bar(Vector2::new(1.0, 1.0), 0.0),
        None
    );
}

#[test]
fn menu_buttons_can_sit_in_the_bottom_bar_with_separators() {
    let screen = Vector2::new(1280.0, 720.0);
    let menu = MainMenuConfig::default()
        .scenery(|s| s.letterbox(|l| l.height(80.0).slide_in(2.0)))
        .buttons_in_bar(true)
        .button_style(|b| b.size(100.0, 30.0))
        .layout(|l| l.row().anchor(Anchor::Center).spacing(20.0))
        .separator(6.0, |p| p);

    let rects = menu.button_rects(screen);
    assert_eq!(rects.len(), 5);
    assert!(rects.iter().all(|r| r.y == 640.0 + 25.0));
    let row_center = (rects[0].x + rects[4].x + rects[4].width) / 2.0;
    assert!((row_center - 640.0).abs() < 0.5);

    let separators = menu.separator_rects(screen);
    assert_eq!(separators.len(), 4);
    assert_eq!(separators[0].x + 3.0, rects[0].x + 100.0 + 10.0);
    assert_eq!(separators[0].y + 3.0, 680.0);
}

#[test]
fn hud_groups_have_their_own_layouts() {
    let screen = Vector2::new(1280.0, 720.0);
    let config = PlayingConfig::default()
        .hud_margin(10.0)
        .hud_button_style(|b| b.size(100.0, 30.0))
        .hud_layout(|l| l.row().anchor(Anchor::Bottom).spacing(0.0))
        .hud_group("top", |l| l.row().anchor(Anchor::TopRight).spacing(0.0))
        .hud_item(HudButton::new("Evidence", Action::Resume).group("top"))
        .hud_item(HudButton::new("Log", Action::Resume))
        .hud_item(HudButton::new("Case", Action::Resume).group("top"))
        .hud_item(HudButton::new("Menu", Action::Resume));

    let rects = config.hud_rects(screen);
    assert_eq!((rects[0].x, rects[0].y), (1070.0, 10.0));
    assert_eq!((rects[2].x, rects[2].y), (1170.0, 10.0));
    assert_eq!((rects[1].x, rects[1].y), (540.0, 680.0));
    assert_eq!((rects[3].x, rects[3].y), (640.0, 680.0));
}

mod weather {
    use vn_engine::frame::scenery::{Scenery, Weather, WeatherKind};
    use vn_engine::raylib::prelude::{Color, Vector2};

    const SCREEN: Vector2 = Vector2::new(1280.0, 720.0);

    fn inside(weather: &Weather, time: f64) -> bool {
        weather.particles(time, SCREEN).all(|p| {
            (0.0..=SCREEN.x).contains(&p.at.x)
                && (0.0..=SCREEN.y).contains(&p.at.y)
                && p.size > 0.0
                && (0.0..=1.0).contains(&p.alpha)
        })
    }

    #[test]
    fn every_particle_stays_on_screen_for_a_long_run() {
        for weather in [Weather::rain(200), Weather::snow(150), Weather::dust(80)] {
            for step in 0..400 {
                let time = step as f64 * 0.37;
                assert!(
                    inside(&weather, time),
                    "{:?} left the screen at {}s",
                    weather.kind,
                    time
                );
            }
        }
    }

    #[test]
    fn the_same_moment_always_looks_the_same() {
        let weather = Weather::snow(64);
        let once: Vec<_> = weather.particles(12.5, SCREEN).collect();
        let again: Vec<_> = weather.particles(12.5, SCREEN).collect();
        assert_eq!(once, again, "nothing is stored, so replay must match");

        let later: Vec<_> = weather.particles(12.6, SCREEN).collect();
        assert_ne!(once, later, "and it still moves");
    }

    #[test]
    fn particles_spread_out_rather_than_falling_in_a_column() {
        let weather = Weather::rain(400);
        let mut columns = [0; 4];
        for particle in weather.particles(3.0, SCREEN) {
            let column = ((particle.at.x / SCREEN.x) * 4.0) as usize;
            columns[column.min(3)] += 1;
        }
        assert!(
            columns.iter().all(|&n| n > 400 / 8),
            "particles bunched up: {:?}",
            columns
        );

        let depths: Vec<f32> = weather.particles(3.0, SCREEN).map(|p| p.size).collect();
        let smallest = depths.iter().cloned().fold(f32::MAX, f32::min);
        let largest = depths.iter().cloned().fold(0.0, f32::max);
        assert!(largest > smallest * 1.5, "no depth variation between drops");
    }

    #[test]
    fn a_count_of_zero_draws_nothing_and_counts_are_capped() {
        assert_eq!(Weather::rain(0).particles(1.0, SCREEN).count(), 0);
        assert_eq!(Weather::snow(10).particles(1.0, SCREEN).count(), 10);
        assert_eq!(
            Weather::dust(99_999).count,
            4000,
            "a typo cannot melt the GPU"
        );
    }

    #[test]
    fn a_stopped_clock_and_absurd_times_still_place_particles() {
        for time in [0.0, -5.0, 1.0e9, f64::INFINITY, f64::NAN] {
            let weather = Weather::snow(32);
            assert!(
                inside(&weather, time),
                "time {} put particles off screen",
                time
            );
        }
    }

    #[test]
    fn scenery_carries_weather_alongside_the_rest() {
        let scenery = Scenery::default()
            .weather(Weather::rain(120).speed(2.0).color(Color::BLUE))
            .vignette(Color::BLACK, 0.3);
        let weather = scenery.weather.expect("weather was set");
        assert_eq!(weather.kind, WeatherKind::Rain);
        assert_eq!(weather.speed, 2.0);
        assert_eq!(weather.color, Color::BLUE);
        assert!(scenery.vignette.is_some(), "and does not replace vignette");
        assert!(Scenery::default().weather.is_none());
    }
}
