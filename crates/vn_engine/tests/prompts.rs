use vn_engine::input::navigation::InputDevice;
use vn_engine::input::pad::{PadButton, PadFamily};
use vn_engine::input::prompts::{Prompt, Prompts};
use vn_engine::raylib::prelude::Vector2;
use vn_engine::request::{Request, Requests};
use vn_engine::ui::cursor::{CursorKind, CursorStyle};

fn table() -> Prompts {
    let mut prompts = Prompts::default();
    prompts.insert("advance", Prompt::new("Space", "A").mouse("click"));
    prompts.insert("log", Prompt::new("L", "Y"));
    prompts
}

#[test]
fn a_prompt_reads_in_the_words_of_whatever_was_last_used() {
    let prompts = table();
    let line = "Press {advance} to go on";
    assert_eq!(
        prompts.fill(line, InputDevice::Keyboard, PadFamily::Xbox),
        "Press Space to go on"
    );
    assert_eq!(
        prompts.fill(line, InputDevice::Gamepad, PadFamily::Xbox),
        "Press A to go on"
    );
    assert_eq!(
        prompts.fill(line, InputDevice::Mouse, PadFamily::Xbox),
        "Press click to go on"
    );
}

#[test]
fn a_prompt_with_no_mouse_wording_falls_back_to_the_keyboards() {
    let prompts = table();
    assert_eq!(
        prompts.fill("{log} opens the log", InputDevice::Mouse, PadFamily::Xbox),
        "L opens the log"
    );
    assert_eq!(
        prompts.fill("{log} opens the log", InputDevice::Gamepad, PadFamily::Xbox),
        "Y opens the log"
    );
}

#[test]
fn an_unknown_token_is_left_alone_rather_than_blanked() {
    let prompts = table();
    assert_eq!(
        prompts.fill(
            "Press {dance} to dance",
            InputDevice::Keyboard,
            PadFamily::Xbox
        ),
        "Press {dance} to dance",
        "a typo should be visible, not silently swallowed"
    );
    assert_eq!(
        prompts.fill("no tokens here", InputDevice::Gamepad, PadFamily::Xbox),
        "no tokens here"
    );
}

#[test]
fn several_tokens_in_one_line_all_follow_the_device() {
    let prompts = table();
    let line = "{advance} to go on, {log} for the log";
    assert_eq!(
        prompts.fill(line, InputDevice::Gamepad, PadFamily::Xbox),
        "A to go on, Y for the log"
    );
}

#[test]
fn a_hand_anywhere_in_the_frame_beats_the_arrow() {
    let mut list = Requests::default();
    list.push(Request::Cursor(CursorKind::Arrow));
    list.push(Request::Cursor(CursorKind::Hand));
    list.push(Request::Cursor(CursorKind::Arrow));
    assert_eq!(
        list.resolve().cursor,
        CursorKind::Hand,
        "one hovered button should win over every quiet one"
    );

    assert_eq!(Requests::default().resolve().cursor, CursorKind::Arrow);
}

#[test]
fn the_cursor_is_sized_in_window_pixels_and_grows_with_the_window() {
    let style = CursorStyle::new("arrow.png")
        .size(28.0)
        .hotspot(0.128, 0.094);
    let natural = Vector2::new(47.0, 64.0);

    let native = style.pixels(CursorKind::Arrow, natural, 1.0);
    assert_eq!(
        native.height, 28,
        "at the design size it is the size asked for"
    );
    assert_eq!(native.width, 21, "and the width keeps the picture's ratio");
    assert_eq!(
        (native.hot_x, native.hot_y),
        (3, 3),
        "the hotspot lands on the tip"
    );

    let four_k = style.pixels(CursorKind::Arrow, natural, 3.0);
    assert_eq!(
        four_k.height, 84,
        "a 4K window gets a cursor three times as tall"
    );
    assert_eq!((four_k.hot_x, four_k.hot_y), (8, 8));
}

#[test]
fn a_cursor_size_stays_within_what_the_os_will_accept() {
    let tiny = CursorStyle::new("a.png").size(1.0).pixels(
        CursorKind::Arrow,
        Vector2::new(10.0, 10.0),
        1.0,
    );
    assert_eq!(tiny.height, 8);
    let huge = CursorStyle::new("a.png").size(400.0).pixels(
        CursorKind::Arrow,
        Vector2::new(10.0, 10.0),
        4.0,
    );
    assert_eq!(huge.height, 256);
    let broken = CursorStyle::new("a.png").size(28.0).pixels(
        CursorKind::Arrow,
        Vector2::new(10.0, 10.0),
        f32::NAN,
    );
    assert_eq!(
        broken.height, 28,
        "a bad scale falls back to the design size"
    );
    let corner = CursorStyle::new("a.png").hotspot(1.0, 1.0).pixels(
        CursorKind::Arrow,
        Vector2::new(10.0, 10.0),
        1.0,
    );
    assert!(
        corner.hot_x < corner.width && corner.hot_y < corner.height,
        "hotspot stays inside"
    );
}

#[test]
fn a_cursor_without_a_hand_picture_keeps_using_the_arrow() {
    let plain = CursorStyle::new("arrow.png");
    assert_eq!(plain.file(CursorKind::Arrow), Some("arrow.png"));
    assert_eq!(plain.file(CursorKind::Hand), Some("arrow.png"));

    let both = CursorStyle::new("arrow.png").hand("hand.png");
    assert_eq!(both.file(CursorKind::Hand), Some("hand.png"));
}

#[test]
fn cursor_files_live_under_ui_unless_a_folder_is_named() {
    assert_eq!(vn_engine::ui::cursor::path("arrow.png"), "ui/arrow.png");
    assert_eq!(
        vn_engine::ui::cursor::path("art/Arrow.PNG"),
        "art/arrow.png"
    );
}

#[test]
fn a_pad_is_recognised_by_the_name_it_reports() {
    for name in [
        "PS5 Controller",
        "DualSense Wireless Controller",
        "Sony Interactive Entertainment Wireless Controller",
        "PS4 Controller",
    ] {
        assert_eq!(PadFamily::detect(name), PadFamily::PlayStation, "{name}");
    }
    for name in ["Nintendo Switch Pro Controller", "Joy-Con (L/R)"] {
        assert_eq!(PadFamily::detect(name), PadFamily::Nintendo, "{name}");
    }
    for name in [
        "Xbox Series Controller",
        "Xbox 360 Controller",
        "8BitDo SN30",
        "",
    ] {
        assert_eq!(
            PadFamily::detect(name),
            PadFamily::Xbox,
            "{name:?}: anything unrecognised gets the most common layout"
        );
    }
}

#[test]
fn a_dualsense_player_is_told_to_press_cross_not_a() {
    let prompts = table();
    let line = "Press {advance} to go on";
    assert_eq!(
        prompts.fill(line, InputDevice::Gamepad, PadFamily::PlayStation),
        "Press Cross to go on"
    );
    assert_eq!(
        prompts.fill(
            "{log} opens the log",
            InputDevice::Gamepad,
            PadFamily::PlayStation
        ),
        "Triangle opens the log"
    );
    assert_eq!(
        prompts.fill(line, InputDevice::Keyboard, PadFamily::PlayStation),
        "Press Space to go on",
        "the pad family only matters while the pad is what is being used"
    );
}

#[test]
fn nintendo_labels_follow_the_button_in_the_same_place() {
    let labels = vn_engine::input::pad::PadLabels::default();
    assert_eq!(
        labels.translate("A", PadFamily::Nintendo),
        "B",
        "the bottom face button is B on a Switch pad"
    );
    assert_eq!(labels.translate("Y", PadFamily::Nintendo), "X");
    assert_eq!(labels.translate("RT", PadFamily::Nintendo), "ZR");
}

#[test]
fn button_words_are_translated_and_everything_else_is_kept() {
    let labels = vn_engine::input::pad::PadLabels::default();
    let ps = PadFamily::PlayStation;
    assert_eq!(labels.translate("RT (hold)", ps), "R2 (hold)");
    assert_eq!(labels.translate("A or X", ps), "Cross or Square");
    assert_eq!(
        labels.translate("D-pad, left stick", ps),
        "D-pad, left stick"
    );
    assert_eq!(labels.translate("Start", ps), "Options");
    assert_eq!(labels.translate("", ps), "");
    assert_eq!(
        labels.translate("Asterisk", ps),
        "Asterisk",
        "only whole words that are button names change"
    );
    assert_eq!(labels.translate("LB / RB", PadFamily::Xbox), "LB / RB");
}

#[test]
fn a_game_with_a_symbol_font_can_use_the_symbols() {
    let mut prompts = table();
    prompts
        .pad_labels
        .set(PadFamily::PlayStation, PadButton::FaceDown, "✕");
    assert_eq!(
        prompts.fill(
            "{advance} to go on",
            InputDevice::Gamepad,
            PadFamily::PlayStation
        ),
        "✕ to go on"
    );
    assert_eq!(
        prompts.fill("{advance} to go on", InputDevice::Gamepad, PadFamily::Xbox),
        "A to go on",
        "an override for one family leaves the others alone"
    );
    assert_eq!(prompts.pad_labels.overrides().collect::<String>(), "✕");
}
