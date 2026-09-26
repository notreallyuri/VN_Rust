use std::rc::Rc;

use novn::data::rollback::RollbackConfig;
use novn::input::navigation::NavigationConfig;
use novn::raylib::prelude::KeyboardKey;
use novn::screens::keybinds::{
    KeySection, KeybindsConfig, KeybindsOverlay, default_keybinds, key_list, key_name,
};
use novn::screens::playing::PlayingConfig;

fn find<'a>(
    sections: &'a [KeySection],
    action: &str,
) -> Option<&'a novn::screens::keybinds::KeyRow> {
    sections
        .iter()
        .flat_map(|section| &section.rows)
        .find(|row| row.action == action)
}

#[test]
fn keys_have_readable_names() {
    assert_eq!(key_name(KeyboardKey::KEY_LEFT_CONTROL), "Ctrl");
    assert_eq!(key_name(KeyboardKey::KEY_PAGE_UP), "Page Up");
    assert_eq!(key_name(KeyboardKey::KEY_F5), "F5");
    assert_eq!(key_name(KeyboardKey::KEY_H), "H");
    assert_eq!(key_name(KeyboardKey::KEY_UP), "Up");
    assert_eq!(
        key_list(&[
            KeyboardKey::KEY_UP,
            KeyboardKey::KEY_DOWN,
            KeyboardKey::KEY_LEFT,
            KeyboardKey::KEY_RIGHT,
            KeyboardKey::KEY_TAB
        ]),
        "Arrow keys, Tab"
    );
    assert_eq!(
        key_list(&[KeyboardKey::KEY_LEFT, KeyboardKey::KEY_RIGHT]),
        "Left, Right"
    );
    assert_eq!(key_name(KeyboardKey::KEY_TAB), "Tab");
    assert_eq!(key_name(KeyboardKey::KEY_KP_ADD), "Kp Add");
    assert_eq!(
        key_list(&[
            KeyboardKey::KEY_LEFT_CONTROL,
            KeyboardKey::KEY_RIGHT_CONTROL,
            KeyboardKey::KEY_A
        ]),
        "Ctrl, A",
        "left and right keys show once"
    );
}

#[test]
fn the_default_list_follows_the_configuration() {
    let playing = PlayingConfig::default();
    let sections = default_keybinds(
        &playing,
        &RollbackConfig::default(),
        &NavigationConfig::default(),
    );
    assert_eq!(sections[0].title, "In the story");
    assert_eq!(sections[1].title, "Menus");

    let hide = find(&sections, "Hide the text").unwrap();
    assert_eq!(hide.keys, "H, middle click");
    assert_eq!(hide.gamepad, "Select");
    assert_eq!(find(&sections, "Skip while held").unwrap().keys, "Ctrl");
    assert_eq!(
        find(&sections, "Roll back").unwrap().keys,
        "Page Up, wheel up"
    );
    assert_eq!(
        find(&sections, "Pause menu").unwrap().keys,
        "Esc, right click"
    );
    assert_eq!(find(&sections, "Quick save").unwrap().keys, "F5");

    let rebound = PlayingConfig::default()
        .keys(|k| {
            k.hide([KeyboardKey::KEY_V])
                .middle_click_hides(false)
                .screenshot([])
        })
        .quick_save_key(None);
    let sections = default_keybinds(
        &rebound,
        &RollbackConfig::default().enabled(false),
        &NavigationConfig::default().gamepad(false),
    );
    let hide = find(&sections, "Hide the text").unwrap();
    assert_eq!(hide.keys, "V");
    assert_eq!(hide.gamepad, "", "no gamepad column without a gamepad");
    assert!(
        find(&sections, "Screenshot").is_none(),
        "unbound actions disappear"
    );
    assert!(find(&sections, "Quick save").is_none());
    assert!(find(&sections, "Roll back").is_none());

    let no_menus = default_keybinds(
        &playing,
        &RollbackConfig::default(),
        &NavigationConfig::default().enabled(false),
    );
    assert_eq!(no_menus.len(), 1);
}

#[test]
fn games_add_or_replace_sections() {
    let defaults = default_keybinds(
        &PlayingConfig::default(),
        &RollbackConfig::default(),
        &NavigationConfig::default(),
    );

    let config = KeybindsConfig::default().section(KeySection::new("Archive").row(
        "E",
        "",
        "Close the evidence",
    ));
    let overlay = KeybindsOverlay::new(Rc::new(config), defaults.clone());
    let titles: Vec<&str> = overlay
        .sections()
        .iter()
        .map(|s| s.title.as_str())
        .collect();
    assert_eq!(titles, ["In the story", "Menus", "Archive", "Anywhere"]);
    assert_eq!(overlay.sections()[3].rows[0].keys, "F1");

    let replaced = KeybindsConfig::default()
        .sections([KeySection::new("Only").row("X", "", "Everything")])
        .open_keys([]);
    let overlay = KeybindsOverlay::new(Rc::new(replaced), defaults);
    let titles: Vec<&str> = overlay
        .sections()
        .iter()
        .map(|s| s.title.as_str())
        .collect();
    assert_eq!(titles, ["Only"], "no F1 row when nothing opens it");
}

#[test]
fn the_controls_stack_into_one_column_when_two_would_cut_labels() {
    let defaults = default_keybinds(
        &PlayingConfig::default(),
        &RollbackConfig::default(),
        &NavigationConfig::default(),
    );
    let overlay = KeybindsOverlay::new(Rc::new(KeybindsConfig::default()), defaults);
    assert_eq!(overlay.columns(1096.0).len(), 2);
    let stacked = overlay.columns(789.0);
    assert_eq!(stacked.len(), 1);
    assert_eq!(stacked[0].len(), overlay.sections().len());
    assert!(overlay.content_height(789.0) > overlay.content_height(1096.0));
}
