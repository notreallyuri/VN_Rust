use std::path::PathBuf;

use novn::data::assets::Assets;
use novn::dev::director::{Kind, candidates, command_line, insert_above, safe};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("novn_director_{}_{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn touch(root: &std::path::Path, file: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, b"").unwrap();
}

const STORY: &str = "scene start:\n  \"one\"\n  choice:\n    \"Go\":\n      \"two\"\n";

#[test]
fn a_line_goes_above_the_current_one_with_its_indentation() {
    let after = insert_above(STORY, 2, "background hall").unwrap();
    assert_eq!(
        after,
        "scene start:\n  background hall\n  \"one\"\n  choice:\n    \"Go\":\n      \"two\"\n"
    );
    let nested = insert_above(STORY, 5, "music night").unwrap();
    assert!(
        nested.contains("\n      music night\n      \"two\""),
        "{nested}"
    );
}

#[test]
fn the_files_own_line_endings_and_last_newline_are_kept() {
    let windows = "scene start:\r\n  \"one\"\r\n";
    assert_eq!(
        insert_above(windows, 2, "sound bell").unwrap(),
        "scene start:\r\n  sound bell\r\n  \"one\"\r\n"
    );
    let no_final_newline = "scene start:\n  \"one\"";
    assert_eq!(
        insert_above(no_final_newline, 2, "sound bell").unwrap(),
        "scene start:\n  sound bell\n  \"one\""
    );
}

#[test]
fn a_line_that_does_not_exist_is_not_written_to() {
    assert_eq!(insert_above(STORY, 0, "sound bell"), None);
    assert_eq!(insert_above(STORY, 99, "sound bell"), None);
}

#[test]
fn an_edit_that_would_break_the_story_is_refused() {
    let fine = insert_above(STORY, 2, "background hall").unwrap();
    assert!(safe(STORY, &fine));

    let inside_choice = insert_above(STORY, 4, "background hall").unwrap();
    assert!(
        !safe(STORY, &inside_choice),
        "a command among a choice's options is an error, so it is not written"
    );
}

#[test]
fn a_story_already_broken_elsewhere_can_still_be_directed() {
    let broken = "scene start:\n  remoe hugo\n  \"one\"\n";
    let after = insert_above(broken, 3, "background hall").unwrap();
    assert!(
        safe(broken, &after),
        "no new error, even though one was already there"
    );
}

#[test]
fn the_lines_read_the_way_a_writer_types_them() {
    assert_eq!(command_line(Kind::Show, "mary tired"), "show mary tired");
    assert_eq!(command_line(Kind::Background, "hall"), "background hall");
    assert_eq!(command_line(Kind::Voice, "line_01"), "voice line_01");
}

#[test]
fn what_can_be_picked_is_what_the_assets_folder_holds() {
    let root = scratch("assets");
    for file in [
        "backgrounds/hall.png",
        "backgrounds/Night.PNG",
        "backgrounds/notes.txt",
        "backgrounds/old/attic.png",
        "backgrounds/not-an-id.png",
        "music/theme.ogg",
        "music/theme.mp3",
        "music/rain.flac",
        "music/storm.OGG",
        "sounds/bell.wav",
        "characters/mary/tired.png",
        "characters/mary/happy.png",
        "characters/hugo/neutral.png",
        "characters/stray.png",
    ] {
        touch(&root, file);
    }
    let assets = Assets::Dir(root);

    assert_eq!(
        candidates(&assets, Kind::Background),
        ["hall"],
        "Night.PNG is left out: the engine looks for night.png, which on a case-sensitive \
         filesystem is a different file, so offering it would write a line that shows a \
         placeholder"
    );
    assert_eq!(
        candidates(&assets, Kind::Music),
        ["rain", "theme"],
        "one entry per track"
    );
    assert_eq!(candidates(&assets, Kind::Sound), ["bell"]);
    assert!(
        candidates(&assets, Kind::Voice).is_empty(),
        "a missing folder is just empty"
    );
    assert_eq!(
        candidates(&assets, Kind::Show),
        ["hugo neutral", "mary happy", "mary tired"]
    );
}

#[test]
fn a_write_puts_the_line_in_the_file() {
    use novn::dev::director::{Written, write_line};
    let root = scratch("write");
    let file = root.join("a.story");
    std::fs::write(&file, STORY).unwrap();
    let written = write_line(file.to_str().unwrap(), 2, "background hall");
    assert!(
        matches!(written, Written::Line { line: 2, .. }),
        "{written:?}"
    );
    assert!(
        std::fs::read_to_string(&file)
            .unwrap()
            .starts_with("scene start:\n  background hall\n  \"one\"")
    );
}

#[test]
fn a_refused_write_leaves_the_file_exactly_as_it_was() {
    use novn::dev::director::{Written, write_line};
    let root = scratch("refused");
    let file = root.join("a.story");
    std::fs::write(&file, STORY).unwrap();
    let written = write_line(file.to_str().unwrap(), 4, "background hall");
    assert_eq!(written, Written::WouldBreak("background hall".into()));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), STORY);
}

#[test]
fn a_story_that_is_not_on_disk_is_not_written() {
    use novn::dev::director::{Written, write_line};
    assert!(matches!(
        write_line("embedded:story/a.story", 2, "background hall"),
        Written::NotOnDisk(_)
    ));
    let root = scratch("short");
    let file = root.join("a.story");
    std::fs::write(&file, STORY).unwrap();
    assert_eq!(
        write_line(file.to_str().unwrap(), 99, "background hall"),
        Written::NoLine
    );
    assert_eq!(std::fs::read_to_string(&file).unwrap(), STORY);
}

#[test]
fn each_kind_previews_what_it_can_and_audio_one_shots_leave_nothing_behind() {
    use novn::dev::director::preview_for;
    assert_eq!(
        preview_for(Kind::Background, "hall").background.as_deref(),
        Some("hall")
    );
    assert_eq!(
        preview_for(Kind::Show, "mary tired").character,
        Some(("mary".into(), "tired".into()))
    );
    assert_eq!(
        preview_for(Kind::Music, "night").music.as_deref(),
        Some("night")
    );
    assert_eq!(preview_for(Kind::Sound, "bell"), Default::default());
    assert_eq!(preview_for(Kind::Voice, "line_01"), Default::default());
}

#[test]
fn closing_the_director_always_clears_the_preview() {
    use novn::dev::director::{DirectorOverlay, preview, preview_for, set_preview};
    let overlay = DirectorOverlay::new();
    set_preview(preview_for(Kind::Background, "hall"));
    drop(overlay);
    assert_eq!(
        preview(),
        Default::default(),
        "a screen change drops the overlay without closing it"
    );
}
