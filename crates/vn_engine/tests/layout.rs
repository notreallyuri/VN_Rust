use vn_engine::raylib::prelude::Vector2;
use vn_engine::{Action, MainMenuConfig};

fn menu(buttons: usize) -> MainMenuConfig {
    (0..buttons).fold(
        MainMenuConfig::default().button_style(|b| b.size(260.0, 52.0)),
        |menu, i| menu.button(format!("{}", i), Action::Quit),
    )
}

#[test]
fn a_short_menu_stays_where_it_was_put() {
    let rects = menu(3).button_rects(Vector2::new(1280.0, 720.0));
    assert_eq!(rects[0].y, 720.0 * 0.45);
    assert_eq!(rects[1].y - rects[0].y, 52.0 + 18.0);
}

#[test]
fn a_long_menu_moves_up_to_keep_its_bottom_margin() {
    let rects = menu(6).button_rects(Vector2::new(1280.0, 720.0));
    let last = rects.last().unwrap();
    assert_eq!(last.y + last.height, 720.0 - 40.0);
    assert!(rects[0].y >= 720.0 * 0.25 + 64.0, "overlaps the title");
    assert_eq!(rects[1].y - rects[0].y, 52.0 + 18.0);
}

#[test]
fn a_menu_too_long_to_move_squeezes_its_spacing() {
    let rects = menu(8).button_rects(Vector2::new(1280.0, 720.0));
    let last = rects.last().unwrap();
    assert!(last.y + last.height <= 720.0 - 40.0 + 0.01);
    assert_eq!(rects[0].y, 720.0 * 0.25 + 64.0);
    assert!(rects[1].y - rects[0].y < 52.0 + 18.0);
}
