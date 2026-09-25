use novn::raylib::prelude::Vector2;
use novn::screens::playing::NvlStyle;

const SCREEN: Vector2 = Vector2::new(1280.0, 720.0);

#[test]
fn the_page_is_centred_and_never_wider_than_its_limit() {
    let style = NvlStyle::default().margin(60.0).max_width(Some(1000.0));
    let rect = style.rect(SCREEN);

    assert_eq!(rect.width, 1000.0);
    assert_eq!(rect.x, 140.0, "centred on the screen");
    assert_eq!(rect.y, 60.0);
    assert_eq!(
        rect.height, 600.0,
        "the margin comes off the top and bottom"
    );

    let narrow = style.rect(Vector2::new(800.0, 720.0));
    assert_eq!(narrow.width, 680.0, "a small window wins over the limit");
    assert_eq!(narrow.x, 60.0);
}

#[test]
fn the_padding_comes_out_of_the_text_area() {
    let style = NvlStyle::default().margin(40.0).padding(30.0);
    let rect = style.rect(SCREEN);
    let area = style.text_area(rect);

    assert_eq!(area.x, rect.x + 30.0);
    assert_eq!(area.y, rect.y + 30.0);
    assert_eq!(area.width, rect.width - 60.0);
    assert_eq!(area.height, rect.height - 60.0);
}

#[test]
fn a_full_page_starts_a_new_one_instead_of_scrolling() {
    let style = NvlStyle::default().entry_spacing(10.0);
    let heights = [100.0, 100.0, 100.0, 100.0];

    assert_eq!(
        style.page_start(&heights, 500.0),
        0,
        "everything fits on one page"
    );
    assert_eq!(
        style.page_start(&heights, 250.0),
        2,
        "the third line opens the second page"
    );
    assert_eq!(
        style.page_start(&heights, 50.0),
        3,
        "a line taller than the page still gets one of its own"
    );
    assert_eq!(style.page_start(&[], 250.0), 0);
}
