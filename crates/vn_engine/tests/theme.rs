use vn_engine::raylib::prelude::Color;
use vn_engine::screens::keybinds::KeybindsConfig;
use vn_engine::screens::log::LogConfig;
use vn_engine::screens::playing::PlayingConfig;
use vn_engine::ui::TextStyle;
use vn_engine::ui::button::ButtonStyle;
use vn_engine::ui::fonts::FontRole;
use vn_engine::ui::theme::Theme;
use vn_engine::ui::toast::ToastConfig;

const OXBLOOD: Color = Color::new(88, 28, 24, 240);
const PARCHMENT: Color = Color::new(226, 206, 168, 255);
const SMOKE: Color = Color::new(8, 6, 5, 200);

fn theme() -> Theme {
    Theme::default()
        .panel(|p| p.color(OXBLOOD))
        .plate(|p| p.color(SMOKE))
        .backdrop(SMOKE)
        .button(|b| b.color(OXBLOOD).font(FontRole::Choice))
        .title(FontRole::Title, PARCHMENT)
        .body(FontRole::Dialogue, PARCHMENT)
        .label(FontRole::Menu, PARCHMENT)
}

#[test]
fn a_theme_paints_a_screen_without_moving_it() {
    let plain = LogConfig::default();
    let themed = LogConfig::default().themed(&theme());

    assert_eq!(themed.panel.color, OXBLOOD);
    assert_eq!(themed.backdrop, SMOKE);
    assert_eq!(themed.title_text.color, PARCHMENT);
    assert_eq!(
        themed.title_text.size, plain.title_text.size,
        "the screen keeps the size it asked for"
    );
    assert_eq!(themed.back_button.color, OXBLOOD);
    assert_eq!(
        (themed.back_button.width, themed.back_button.height),
        (plain.back_button.width, plain.back_button.height),
        "and the geometry of its buttons"
    );
    assert_eq!(
        themed.panel_width, plain.panel_width,
        "a theme has no opinion about layout"
    );
}

#[test]
fn a_theme_with_no_opinion_changes_nothing() {
    let plain = LogConfig::default();
    let same = LogConfig::default().themed(&Theme::default());

    assert_eq!(same.panel, plain.panel);
    assert_eq!(same.backdrop, plain.backdrop);
    assert_eq!(same.title_text, plain.title_text);
    assert_eq!(same.back_button, plain.back_button);

    let toast = ToastConfig::default().themed(&Theme::default());
    assert_eq!(toast.panel, ToastConfig::default().panel);
    assert_eq!(toast.text, ToastConfig::default().text);
}

#[test]
fn every_panel_of_the_playing_screen_is_painted() {
    let themed = PlayingConfig::default()
        .dialogue_box(|b| b.name_plate(|n| n))
        .themed(&theme());

    assert_eq!(themed.dialogue_box.panel.color, OXBLOOD);
    assert_eq!(themed.choice_preview.panel.color, OXBLOOD);
    assert_eq!(themed.nvl.panel.color, OXBLOOD);
    assert_eq!(themed.indicator.color, SMOKE, "the indicator is a plate");
    assert_eq!(
        themed.dialogue_box.name_plate.unwrap().panel.color,
        SMOKE,
        "and so is the name plate"
    );
    assert_eq!(themed.dialogue_text.color, PARCHMENT);
    assert_eq!(themed.choice_button.color, OXBLOOD);
    assert_eq!(
        themed.choice_button.width,
        PlayingConfig::default().choice_button.width
    );
}

#[test]
fn a_styled_button_keeps_what_the_screen_set_and_takes_the_rest() {
    let role = ButtonStyle::default()
        .color(OXBLOOD)
        .font(FontRole::Choice)
        .text_color(PARCHMENT);
    let theirs = ButtonStyle::default().size(72.0, 28.0).padding(30.0, 6.0);
    let themed = theirs.themed(&role);

    assert_eq!((themed.width, themed.height), (72.0, 28.0), "its geometry");
    assert_eq!(themed.padding.x, 30.0);
    assert_eq!(themed.text.font, FontRole::Choice, "the theme's label");
    assert_eq!(themed.text.color, PARCHMENT);
    assert_eq!(themed.color, OXBLOOD);
    assert_eq!(
        themed.text.size, role.text.size,
        "a button's label size is part of the theme, so buttons match each other; a          screen that wants another size sets it after"
    );
}

#[test]
fn a_screen_can_still_disagree_with_the_theme() {
    let themed = KeybindsConfig::default()
        .themed(&theme())
        .action_text(TextStyle::new(FontRole::Menu, 16.0, Color::RED));

    assert_eq!(
        themed.action_text.color,
        Color::RED,
        "what a screen sets after the theme wins"
    );
    assert_eq!(themed.title_text.color, PARCHMENT);
}

#[test]
#[should_panic(expected = "`theme` is what the default screens start from")]
fn a_theme_set_after_a_screen_says_so() {
    let _ = vn_engine::app::VnApp::new("Test")
        .log(|l| l.title("Log"))
        .theme(|t| t.backdrop(SMOKE));
}
