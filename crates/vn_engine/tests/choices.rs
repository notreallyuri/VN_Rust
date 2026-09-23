use vn_engine::data::resources::{choice_path, preview_path};
use vn_engine::raylib::prelude::{Color, Rectangle, Vector2};
use vn_engine::screens::playing::{ChoiceImageStyle, ChoicePreviewStyle, PlayingConfig};
use vn_engine::ui::button::IconSide;
use vn_engine::ui::layout::Anchor;

const SCREEN: Vector2 = Vector2::new(1280.0, 720.0);

fn option(text: &str, index: usize) -> vn_script::ChoiceOption {
    vn_script::ChoiceOption::new(text, index)
}

#[test]
fn an_options_picture_becomes_the_buttons_fill_or_its_icon() {
    let mut chosen = option("The north road", 2);
    chosen.image = Some("north".into());

    let filled = PlayingConfig::default().option_style(&chosen);
    assert_eq!(
        filled.image.as_ref().map(|image| image.path.as_str()),
        Some(choice_path("north").as_str())
    );
    assert!(filled.icon.is_none());

    let config =
        PlayingConfig::default().choice_image(|i| i.icon().size(40.0).side(IconSide::Right));
    let iconed = config.option_style(&chosen);
    assert!(iconed.image.is_none());
    let icon = iconed.icon.expect("an icon");
    assert_eq!(icon.path, choice_path("north"));
    assert_eq!(icon.size, 40.0);
    assert_eq!(icon.side, IconSide::Right);

    let plain = PlayingConfig::default().option_style(&option("Turn back", 0));
    assert!(plain.image.is_none() && plain.icon.is_none());
}

#[test]
fn a_game_styling_its_choices_keeps_that_style_under_the_picture() {
    let mut chosen = option("The north road", 1);
    chosen.image = Some("north".into());

    let config = PlayingConfig::default()
        .choice_button_for(|index, _, style| style.color(Color::new(index as u8, 0, 0, 255)));
    let style = config.option_style(&chosen);

    assert_eq!(style.color, Color::new(1, 0, 0, 255), "styled by its index");
    assert!(style.image.is_some());
}

#[test]
fn both_pictures_of_every_option_are_loaded() {
    let mut first = option("The north road", 0);
    first.image = Some("north".into());
    first.preview = Some("north_view".into());
    let second = option("Turn back", 1);

    assert_eq!(
        PlayingConfig::default().option_pictures(&[first, second]),
        [choice_path("north"), preview_path("north_view")]
    );
}

#[test]
fn a_preview_keeps_its_aspect_inside_the_panel() {
    let style = ChoicePreviewStyle::default()
        .size(400.0, 300.0)
        .anchor(Anchor::TopRight)
        .margin(40.0)
        .padding(10.0);

    let panel = style.rect(SCREEN);
    assert_eq!(panel.width, 400.0);
    assert_eq!(panel.x + panel.width, SCREEN.x - 40.0, "against the margin");
    assert_eq!(panel.y, 40.0);

    let wide = style.picture_rect(panel, Vector2::new(1600.0, 900.0));
    assert_eq!(wide.width, 380.0, "the width fills the padded panel");
    assert!((wide.height - 213.75).abs() < 0.01);
    assert!((wide.y - (panel.y + 10.0 + (280.0 - 213.75) / 2.0)).abs() < 0.01);

    let missing = style.picture_rect(panel, Vector2::zero());
    assert_eq!(
        missing,
        Rectangle::new(panel.x + 10.0, panel.y + 10.0, 380.0, 280.0)
    );
}

#[test]
fn an_image_style_is_built_the_same_way_as_the_rest() {
    let style = ChoiceImageStyle::default().icon().gap(4.0).tint(Color::RED);
    assert_eq!(style.gap, 4.0);
    assert_eq!(style.tint, Color::RED);
}
