use std::fs;
use std::path::PathBuf;

use vn_engine::{Audio, AudioConfig, Fade, music_path, sound_path};

fn temp_assets() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vn_engine_audio_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("music")).unwrap();
    fs::create_dir_all(dir.join("sounds")).unwrap();
    dir
}

#[test]
fn audio_files_are_found_by_id_with_any_supported_extension() {
    let assets = temp_assets();
    fs::write(assets.join("music/theme.mp3"), b"").unwrap();
    fs::write(assets.join("music/theme.ogg"), b"").unwrap();
    fs::write(assets.join("sounds/knock.wav"), b"").unwrap();

    assert_eq!(
        music_path(&assets, "theme"),
        Some(assets.join("music/theme.ogg")),
        "ogg is tried first"
    );
    assert_eq!(
        sound_path(&assets, "knock"),
        Some(assets.join("sounds/knock.wav"))
    );
    assert_eq!(music_path(&assets, "knock"), None);
    assert_eq!(sound_path(&assets, "missing"), None);
    fs::remove_dir_all(&assets).unwrap();
}

#[test]
fn fades_move_towards_their_target_at_a_fixed_rate() {
    let mut fade = Fade::fade_in();
    fade.step(0.25, 1.0);
    assert_eq!(fade.gain, 0.25);
    fade.step(1.0, 1.0);
    assert_eq!(fade.gain, 1.0, "never overshoots");

    fade.target = 0.0;
    assert!(!fade.silent());
    fade.step(0.5, 2.0);
    assert_eq!(fade.gain, 0.75);
    fade.step(10.0, 2.0);
    assert!(fade.silent());

    let mut instant = Fade::fade_in();
    instant.step(0.0, 0.0);
    assert_eq!(instant.gain, 1.0);
}

#[test]
fn silent_audio_still_tracks_the_wanted_music() {
    let mut audio = Audio::silent();
    assert!(!audio.is_enabled());
    assert_eq!(audio.music(), None);

    audio.play_music(Some("theme"));
    assert_eq!(audio.music(), Some("theme"));
    audio.update(0.1);
    audio.play_sound("knock");
    audio.play_music(None);
    assert_eq!(audio.music(), None);
}

#[test]
fn audio_config_builder() {
    let config = AudioConfig::default()
        .menu_music("title")
        .fade_seconds(-1.0);
    assert!(config.enabled);
    assert_eq!(config.menu_music.as_deref(), Some("title"));
    assert_eq!(config.fade_seconds, 0.0);
    assert!(!AudioConfig::default().enabled(false).enabled);
}
