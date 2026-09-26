use std::fs;
use std::path::PathBuf;

use novn::data::settings::{Settings, SettingsStore};
use novn::screen::ScreenState;
use novn::screen_manager::close_needs_confirmation;
use novn::screens::playing::Typewriter;
use novn::screens::settings::SettingsConfig;
use novn::script::StoryVm;

fn temp_file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vn_engine_settings_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let _ = fs::remove_file(&path);
    path
}

#[test]
fn missing_or_broken_settings_use_the_defaults() {
    let path = temp_file("missing.json");
    assert_eq!(Settings::load(&path), Settings::default());

    fs::write(&path, "not json").unwrap();
    assert_eq!(Settings::load(&path), Settings::default());

    fs::write(&path, r#"{ "fullscreen": true }"#).unwrap();
    assert_eq!(
        Settings::load(&path),
        Settings {
            fullscreen: true,
            ..Settings::default()
        }
    );
    fs::remove_file(&path).unwrap();
}

#[test]
fn changes_are_saved_right_away() {
    let path = temp_file("store.json");

    let mut store = SettingsStore::load(&path);
    store.update(|_| {});
    assert!(!path.exists(), "unchanged settings aren't written");

    store.update(|s| s.text_speed = 80);
    assert_eq!(SettingsStore::load(&path).values.text_speed, 80);

    let mut memory = SettingsStore::in_memory();
    memory.update(|s| s.fullscreen = true);
    assert!(memory.values.fullscreen);
    fs::remove_file(&path).unwrap();
}

#[test]
fn typewriter_reveals_characters_over_time() {
    let text = "Héllo, world";
    let mut typewriter = Typewriter::start(text, 10, 100.0);

    assert_eq!(typewriter.visible(100.0), 0);
    assert_eq!(typewriter.visible(100.35), 3);
    assert!(!typewriter.is_done(100.5));
    assert_eq!(typewriter.visible(200.0), 12);
    assert!(typewriter.is_done(200.0));

    let mut skipped = Typewriter::start(text, 10, 100.0);
    skipped.finish();
    assert_eq!(skipped.visible(100.0), 12);

    typewriter = Typewriter::start(text, 0, 100.0);
    assert!(typewriter.is_done(100.0));
    assert_eq!(Typewriter::finished(text).visible(0.0), 12);
}

#[test]
fn text_speeds_cycle() {
    let config = SettingsConfig::default();
    assert_eq!(config.next_text_speed(20), 40);
    assert_eq!(config.next_text_speed(80), 0);
    assert_eq!(config.next_text_speed(0), 20);
    assert_eq!(config.next_text_speed(33), 20);
    assert_eq!(config.text_speed_name(0), "Instant");
    assert_eq!(config.text_speed_name(33), "33 chars/s");

    let custom = SettingsConfig::default().text_speeds([("Calm", 15), ("Brisk", 60)]);
    assert_eq!(custom.next_text_speed(60), 15);
}

#[test]
fn closing_asks_only_during_a_game() {
    let mut story = StoryVm::from_source("scene start:\n  \"x\"\n");
    let asks = |state: ScreenState, story: &StoryVm| close_needs_confirmation(&state, story);

    assert!(!asks(ScreenState::StartScreen, &story));
    assert!(!asks(ScreenState::MainMenu, &story));
    assert!(asks(ScreenState::Playing, &story));
    assert!(asks(ScreenState::TextInput, &story));
    assert!(!asks(ScreenState::Settings, &story), "no game started yet");
    assert!(!asks(ScreenState::Load, &story));

    story.advance_until_blocking();
    assert!(asks(ScreenState::Settings, &story));
    assert!(asks(ScreenState::Custom("inventory".into()), &story));
    assert!(!asks(ScreenState::MainMenu, &story));

    story.advance_until_blocking();
    assert!(!asks(ScreenState::Playing, &story), "the story is over");
    assert!(!asks(ScreenState::Settings, &story));
}

#[test]
fn sliders_map_positions_to_settings() {
    use novn::screens::settings::SettingsRow::*;

    let config = SettingsConfig::default();
    let mut settings = Settings::default();

    config.set_fraction(MusicVolume, &mut settings, 0.42);
    assert_eq!(settings.music_volume, 40, "snapped to 5% steps");
    config.set_fraction(SoundVolume, &mut settings, 1.3);
    assert_eq!(settings.sound_volume, 100);
    config.set_fraction(SoundVolume, &mut settings, 0.0);
    assert_eq!(config.value_name(SoundVolume, &settings), "Off");
    assert_eq!(config.fraction(MusicVolume, &settings), 0.4);

    config.set_fraction(TextSpeed, &mut settings, 0.0);
    assert_eq!(settings.text_speed, 20);
    config.set_fraction(TextSpeed, &mut settings, 0.4);
    assert_eq!(settings.text_speed, 40, "the nearest of four stops");
    config.set_fraction(TextSpeed, &mut settings, 1.0);
    assert_eq!(settings.text_speed, 0);
    assert_eq!(config.value_name(TextSpeed, &settings), "Instant");
    assert_eq!(config.fraction(TextSpeed, &settings), 1.0);

    let custom = SettingsConfig::default().volume_step(10);
    assert_eq!(custom.volume_at(0.44), 40);
    assert_eq!(custom.volume_at(0.46), 50);
}

#[test]
fn arrow_keys_step_sliders_and_stop_at_the_ends() {
    use novn::screens::settings::SettingsRow::*;

    let config = SettingsConfig::default();
    let mut settings = Settings {
        music_volume: 72,
        ..Settings::default()
    };

    config.step(MusicVolume, &mut settings, 1);
    assert_eq!(
        settings.music_volume, 75,
        "an odd value snaps before stepping"
    );
    config.step(MusicVolume, &mut settings, -1);
    assert_eq!(settings.music_volume, 70);
    settings.sound_volume = 100;
    config.step(SoundVolume, &mut settings, 1);
    assert_eq!(settings.sound_volume, 100);

    settings.text_speed = 80;
    config.step(TextSpeed, &mut settings, 1);
    assert_eq!(settings.text_speed, 0);
    config.step(TextSpeed, &mut settings, 1);
    assert_eq!(settings.text_speed, 0, "no wrap-around at the end");
    config.step(TextSpeed, &mut settings, -3);
    assert_eq!(settings.text_speed, 20);

    config.step(Display, &mut settings, 1);
    assert!(settings.fullscreen);
}

#[test]
fn rows_and_old_settings_files() {
    use novn::screens::settings::SettingsRow::*;

    assert_eq!(
        SettingsConfig::default().rows(),
        [
            Display,
            TextSpeed,
            MusicVolume,
            SoundVolume,
            VoiceVolume,
            AutoDelay,
            SkipUnseen,
            Accessibility
        ]
    );
    assert_eq!(
        SettingsConfig::default()
            .voice_row(false)
            .play_rows(false)
            .rows(),
        [Display, TextSpeed, MusicVolume, SoundVolume, Accessibility]
    );
    assert_eq!(
        SettingsConfig::default()
            .audio_rows(false)
            .play_rows(false)
            .accessibility_rows(false)
            .rows(),
        [Display, TextSpeed]
    );
    assert!(!Accessibility.is_slider() && !ReduceMotion.is_slider());
    assert!(!SkipUnseen.is_slider() && AutoDelay.is_slider());
    assert!(TextSpeed.is_slider() && !Display.is_slider());

    let old: Settings =
        serde_json::from_str(r#"{ "fullscreen": false, "text_speed": 40 }"#).unwrap();
    assert_eq!(old.music_volume, 70);
    assert_eq!(old.sound_volume, 80);
    assert_eq!(old.music_gain(), 0.7);
    assert_eq!(old.voice_volume, 100);
    assert_eq!(old.auto_delay, 1500);
    assert!(!old.skip_unseen);
    assert!(
        !old.reduce_motion,
        "a settings file from before it existed reads it as off"
    );
    let loud = Settings {
        music_volume: 250,
        ..Settings::default()
    };
    assert_eq!(loud.music_gain(), 1.0);
}

#[test]
fn slider_math() {
    use novn::raylib::prelude::Rectangle;
    use novn::ui::{slider_fraction, slider_step};

    let track = Rectangle::new(100.0, 0.0, 200.0, 20.0);
    assert_eq!(slider_fraction(track, 50.0), 0.0);
    assert_eq!(slider_fraction(track, 150.0), 0.25);
    assert_eq!(slider_fraction(track, 400.0), 1.0);
    assert_eq!(
        slider_fraction(Rectangle::new(0.0, 0.0, 0.0, 0.0), 5.0),
        0.0
    );

    assert_eq!(slider_step(0.0, 4), 0);
    assert_eq!(slider_step(0.49, 4), 1);
    assert_eq!(slider_step(0.51, 4), 2);
    assert_eq!(slider_step(1.0, 4), 3);
    assert_eq!(slider_step(0.7, 1), 0);
}

#[test]
fn tooltips_wait_for_the_delay_and_hide_on_click() {
    use novn::ui::tooltip::TooltipTimer;

    let mut timer = TooltipTimer::default();
    timer.update(Some("Save".into()), 10.0, false);
    assert_eq!(timer.visible(10.2, 0.5), None);
    timer.update(Some("Save".into()), 10.4, false);
    assert_eq!(timer.visible(10.6, 0.5), Some("Save"));

    timer.update(Some("Load".into()), 10.7, false);
    assert_eq!(
        timer.visible(10.8, 0.5),
        None,
        "a new target restarts the delay"
    );

    timer.update(Some("Load".into()), 11.3, true);
    assert_eq!(timer.visible(11.4, 0.5), None, "clicking hides it");
    timer.update(None, 11.5, false);
    timer.update(Some("Load".into()), 11.6, false);
    assert_eq!(
        timer.visible(12.2, 0.5),
        Some("Load"),
        "until the pointer comes back"
    );

    timer.update(None, 12.3, false);
    assert_eq!(timer.visible(20.0, 0.5), None);
}

#[test]
fn auto_delay_and_skip_rows() {
    use novn::screens::settings::SettingsRow::*;

    let config = SettingsConfig::default();
    let mut settings = Settings::default();

    assert_eq!(config.fraction(AutoDelay, &settings), 2.0 / 9.0);
    config.set_fraction(AutoDelay, &mut settings, 1.0);
    assert_eq!(settings.auto_delay, 5000);
    config.set_fraction(AutoDelay, &mut settings, 0.45);
    assert_eq!(settings.auto_delay, 2500);
    assert_eq!(config.value_name(AutoDelay, &settings), "2.5 s");
    config.step(AutoDelay, &mut settings, -9);
    assert_eq!(settings.auto_delay, 500, "stops at the shortest");

    assert_eq!(config.value_name(SkipUnseen, &settings), "Seen text");
    config.step(SkipUnseen, &mut settings, 1);
    assert!(settings.skip_unseen);
    assert_eq!(config.value_name(SkipUnseen, &settings), "All text");

    config.set_fraction(VoiceVolume, &mut settings, 0.33);
    assert_eq!(settings.voice_volume, 35);
    assert_eq!(settings.voice_gain(), 0.35);
}

#[test]
fn the_accessibility_page_has_its_own_rows_and_title() {
    use novn::screens::settings::{SettingsPage, SettingsRow};
    let config = SettingsConfig::default();
    assert_eq!(
        config.page_rows(SettingsPage::Accessibility),
        [
            SettingsRow::TextSize,
            SettingsRow::UiScale,
            SettingsRow::TextBackdrop,
            SettingsRow::TextOutline,
            SettingsRow::ReduceMotion
        ]
    );
    assert_eq!(config.page_rows(SettingsPage::Main), config.rows());
    assert_eq!(
        config.title_of(SettingsPage::Accessibility),
        "Accessibility"
    );

    let mut settings = Settings::default();
    config.step(SettingsRow::ReduceMotion, &mut settings, 1);
    assert!(settings.reduce_motion);
    assert_eq!(
        config.value_name(SettingsRow::ReduceMotion, &settings),
        "On"
    );
    config.step(SettingsRow::Accessibility, &mut settings, 1);
    assert!(
        settings.reduce_motion,
        "opening the page changes no setting"
    );
}

#[test]
fn text_size_steps_through_its_sizes_and_stops_at_the_ends() {
    use novn::screens::settings::SettingsRow;
    let config = SettingsConfig::default();
    let mut settings = Settings::default();
    assert_eq!(settings.text_size, 100);
    config.step(SettingsRow::TextSize, &mut settings, 1);
    assert_eq!(settings.text_size, 115);
    assert_eq!(config.value_name(SettingsRow::TextSize, &settings), "115%");
    for _ in 0..10 {
        config.step(SettingsRow::TextSize, &mut settings, 1);
    }
    assert_eq!(settings.text_size, 150);
    for _ in 0..10 {
        config.step(SettingsRow::TextSize, &mut settings, -1);
    }
    assert_eq!(settings.text_size, 90);
    config.set_fraction(SettingsRow::TextSize, &mut settings, 1.0);
    assert_eq!(settings.text_size, 150);

    let old: Settings = serde_json::from_str(r#"{ "fullscreen": false }"#).unwrap();
    assert_eq!(
        old.text_size, 100,
        "a file from before it existed reads as normal size"
    );
}

#[test]
fn interface_size_cycles_through_its_scales() {
    use novn::screens::settings::SettingsRow;
    let config = SettingsConfig::default();
    let mut settings = Settings::default();
    assert_eq!(settings.ui_scale, 100);
    assert!(!SettingsRow::UiScale.is_slider());
    let mut seen = Vec::new();
    for _ in 0..4 {
        config.step(SettingsRow::UiScale, &mut settings, 1);
        seen.push(config.value_name(SettingsRow::UiScale, &settings));
    }
    assert_eq!(seen, ["110%", "125%", "150%", "100%"]);
    config.step(SettingsRow::UiScale, &mut settings, -1);
    assert_eq!(
        settings.ui_scale, 150,
        "Left from the first scale wraps to the last"
    );

    let old: Settings = serde_json::from_str(r#"{ "fullscreen": false }"#).unwrap();
    assert_eq!(
        old.ui_scale, 100,
        "a file from before it existed reads as 100"
    );
}

#[test]
fn every_settings_page_fits_above_back_at_every_interface_size() {
    use novn::frame::viewport::{UI_SCALES, scaled_layout};
    use novn::screens::settings::SettingsPage;
    use raylib::prelude::Vector2;

    let config = SettingsConfig {
        self_voicing_row: true,
        ..Default::default()
    };
    for scale in UI_SCALES {
        let (width, height) = scaled_layout((1280, 720), scale);
        let screen = Vector2::new(width as f32, height as f32);
        let back = config.back_rect(screen);
        for page in [SettingsPage::Main, SettingsPage::Accessibility] {
            let count = config.page_rows(page).len();
            let view = config.rows_view(count, screen);
            assert!(
                view.y + view.height <= back.y,
                "{scale}%: {page:?} rows reach the Back button"
            );
            assert!(
                view.height >= 2.0 * config.value_button.height,
                "{scale}%: {page:?} shows fewer than two rows"
            );
            if config.shows_sample(count, screen) {
                let sample = config.sample_rect(count, screen);
                assert!(sample.y >= view.y + view.height);
                assert!(
                    sample.y + sample.height <= back.y,
                    "{scale}%: {page:?} sample covers Back"
                );
            }
        }
    }
}
