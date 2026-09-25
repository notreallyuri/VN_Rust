use novn::game::characters::{Character, Characters};
use novn::raylib::prelude::{Color, Vector2};
use novn::screens::playing::{BustSide, DialogueBoxStyle};

const SCREEN: Vector2 = Vector2::new(1280.0, 720.0);

fn cast() -> Characters {
    let mut characters = Characters::default();
    characters.insert("mary", Character::new("Mary").bust("mary.png"));
    characters.insert(
        "registrar",
        Character::new("The Registrar")
            .bust("busts/registrar.png")
            .box_style(|b| b.height(220.0).color(Color::new(40, 10, 10, 240))),
    );
    characters.insert("hugo", Character::new("Hugo"));
    characters
}

#[test]
fn a_character_can_restyle_the_box_without_losing_the_games_base() {
    let base = DialogueBoxStyle::default().padding(30.0).margin(50.0);
    let cast = cast();

    let plain = cast.dialogue_box(Some("hugo"), &base);
    assert_eq!(
        plain, base,
        "a character with no style of its own changes nothing"
    );
    assert_eq!(
        cast.dialogue_box(None, &base),
        base,
        "and so does narration"
    );

    let styled = cast.dialogue_box(Some("registrar"), &base);
    assert_eq!(styled.height, 220.0, "the override applies");
    assert_eq!(
        styled.padding, 30.0,
        "and the game's base is kept underneath"
    );
    assert_eq!(styled.margin, 50.0);
    assert_eq!(base.height, 170.0, "the base itself is untouched");
}

#[test]
fn a_bust_reserves_room_and_the_text_moves_out_of_its_way() {
    let plain = DialogueBoxStyle::default();
    let with_bust = DialogueBoxStyle::default().bust(|b| b.width(160.0).gap(20.0));
    let rect = plain.rect(SCREEN);

    let open = plain.text_area(rect);
    let squeezed = with_bust.text_area(rect);
    assert_eq!(
        open.width - squeezed.width,
        200.0,
        "the bust and both gaps come out of the text"
    );
    assert_eq!(
        squeezed.x - open.x,
        200.0,
        "and a left bust pushes the text right"
    );

    let right =
        DialogueBoxStyle::default().bust(|b| b.width(160.0).gap(20.0).side(BustSide::Right));
    let other = right.text_area(rect);
    assert_eq!(other.x, open.x, "a right bust leaves the text where it was");
    assert_eq!(other.width, squeezed.width, "but takes the same room");
}

#[test]
fn a_bust_keeps_its_aspect_ratio_and_sits_on_the_boxs_floor() {
    let style = DialogueBoxStyle::default().bust(|b| b.width(200.0).gap(10.0));
    let rect = style.rect(SCREEN);
    let bust = style.bust.clone().unwrap();

    let tall = bust.rect(rect, Vector2::new(300.0, 600.0));
    assert!(
        (tall.width / tall.height - 0.5).abs() < 0.001,
        "portrait keeps its ratio"
    );
    assert!(
        tall.width <= 200.0,
        "and never exceeds the width it was given"
    );
    assert_eq!(
        tall.y + tall.height,
        rect.y + rect.height,
        "a bust stands on the box's floor"
    );
    assert_eq!(tall.x, rect.x + 10.0, "inset by the gap");

    let wide = bust.rect(rect, Vector2::new(600.0, 300.0));
    assert!(
        (wide.width / wide.height - 2.0).abs() < 0.001,
        "landscape does too"
    );
}

#[test]
fn rise_lifts_the_bust_above_the_box_and_sink_drops_it_below() {
    let rect = DialogueBoxStyle::default().rect(SCREEN);
    let square = Vector2::new(400.0, 400.0);

    let flat = DialogueBoxStyle::default().bust(|b| b.width(400.0));
    let lifted = DialogueBoxStyle::default().bust(|b| b.width(400.0).rise(60.0));
    let sunk = DialogueBoxStyle::default().bust(|b| b.width(400.0).sink(40.0));

    let base = flat.bust.clone().unwrap().rect(rect, square);
    let up = lifted.bust.clone().unwrap().rect(rect, square);
    let down = sunk.bust.clone().unwrap().rect(rect, square);

    assert!(up.y < base.y, "rise lifts the top above the box");
    assert_eq!(
        down.y + down.height,
        rect.y + rect.height + 40.0,
        "sink pushes the feet below the box"
    );
}

#[test]
fn a_box_with_no_bust_uses_its_whole_width() {
    let style = DialogueBoxStyle::default().padding(24.0);
    let rect = style.rect(SCREEN);
    let area = style.text_area(rect);
    assert_eq!(area.x, rect.x + 24.0);
    assert_eq!(area.width, rect.width - 48.0);
    assert_eq!(area.y, rect.y + 24.0);
}

#[test]
fn a_character_without_a_bust_is_not_given_one() {
    let cast = cast();
    assert_eq!(cast.bust("mary"), Some("mary.png"));
    assert_eq!(cast.bust("hugo"), None);
    assert_eq!(cast.bust("nobody"), None);
}
