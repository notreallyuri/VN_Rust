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

mod layout {
    use vn_engine::raylib::prelude::{Rectangle, Vector2};
    use vn_engine::{Action, Align, Anchor, Layout, PlayingConfig};

    fn area() -> Rectangle {
        Rectangle::new(0.0, 0.0, 1000.0, 600.0)
    }

    fn sizes(count: usize) -> Vec<Vector2> {
        vec![Vector2::new(200.0, 50.0); count]
    }

    fn corners(rects: &[Rectangle]) -> Vec<(f32, f32)> {
        rects.iter().map(|r| (r.x, r.y)).collect()
    }

    #[test]
    fn a_column_is_centered_by_default() {
        let rects = Layout::default().spacing(10.0).place(area(), &sizes(3));
        assert_eq!(
            corners(&rects),
            [(400.0, 215.0), (400.0, 275.0), (400.0, 335.0)]
        );
    }

    #[test]
    fn a_row_anchored_top_right() {
        let rects = Layout::default()
            .row()
            .anchor(Anchor::TopRight)
            .spacing(10.0)
            .place(area(), &sizes(3));
        assert_eq!(corners(&rects), [(380.0, 0.0), (590.0, 0.0), (800.0, 0.0)]);
    }

    #[test]
    fn a_two_column_grid_fills_rows_first() {
        let layout = Layout::default()
            .grid(2)
            .anchor(Anchor::TopLeft)
            .spacing_xy(20.0, 10.0);
        let rects = layout.place(area(), &sizes(5));
        assert_eq!(
            corners(&rects),
            [
                (0.0, 0.0),
                (220.0, 0.0),
                (0.0, 60.0),
                (220.0, 60.0),
                (0.0, 120.0)
            ]
        );
        assert_eq!(layout.rows(5), 3);
        assert_eq!(layout.block_size(&sizes(5)), Vector2::new(420.0, 170.0));
    }

    #[test]
    fn a_grid_with_fewer_items_than_columns_is_one_row() {
        let layout = Layout::default().grid(4);
        assert_eq!(layout.columns(2), 2);
        assert_eq!(layout.block_size(&sizes(2)), Vector2::new(416.0, 50.0));
    }

    #[test]
    fn items_of_different_widths_align_in_their_column() {
        let sizes = [Vector2::new(300.0, 50.0), Vector2::new(100.0, 50.0)];
        let x = |align| {
            Layout::default()
                .align(align)
                .anchor(Anchor::TopLeft)
                .place(area(), &sizes)[1]
                .x
        };
        assert_eq!(x(Align::Start), 0.0);
        assert_eq!(x(Align::Center), 100.0);
        assert_eq!(x(Align::End), 200.0);
    }

    #[test]
    fn spacing_shrinks_when_the_block_does_not_fit() {
        let small = Rectangle::new(0.0, 0.0, 1000.0, 160.0);
        let rects = Layout::default().spacing(20.0).place(small, &sizes(3));
        assert_eq!(rects[1].y - rects[0].y, 55.0);
        assert_eq!(rects[2].y + rects[2].height, 160.0);
    }

    #[test]
    fn every_anchor() {
        let place = |anchor| {
            let r = Layout::default().anchor(anchor).place(area(), &sizes(1))[0];
            (r.x, r.y)
        };
        assert_eq!(place(Anchor::TopLeft), (0.0, 0.0));
        assert_eq!(place(Anchor::Top), (400.0, 0.0));
        assert_eq!(place(Anchor::Left), (0.0, 275.0));
        assert_eq!(place(Anchor::Center), (400.0, 275.0));
        assert_eq!(place(Anchor::Right), (800.0, 275.0));
        assert_eq!(place(Anchor::BottomLeft), (0.0, 550.0));
        assert_eq!(place(Anchor::Bottom), (400.0, 550.0));
        assert_eq!(place(Anchor::BottomRight), (800.0, 550.0));
    }

    #[test]
    fn custom_arrangements_get_the_area_and_sizes() {
        let layout = Layout::default().custom(|area, sizes| {
            sizes
                .iter()
                .enumerate()
                .map(|(i, s)| Rectangle::new(area.x + i as f32 * 5.0, area.y, s.x, s.y))
                .collect()
        });
        let rects = layout.place(Rectangle::new(10.0, 20.0, 100.0, 100.0), &sizes(2));
        assert_eq!(corners(&rects), [(10.0, 20.0), (15.0, 20.0)]);
    }

    #[test]
    fn nothing_to_place() {
        assert!(Layout::default().grid(3).place(area(), &[]).is_empty());
    }

    #[test]
    fn playing_screen_defaults_match_the_old_positions() {
        let screen = Vector2::new(1280.0, 720.0);
        let config = PlayingConfig::default()
            .hud_button("A", Action::Quit)
            .hud_button("B", Action::Quit);

        let hud = config.hud_rects(screen);
        assert_eq!(corners(&hud), [(994.0, 16.0), (1134.0, 16.0)]);

        let choices = config.choice_rects(3, screen);
        let total = 3.0 * 56.0 + 2.0 * 16.0;
        assert_eq!(choices[0].x, (1280.0 - 720.0) / 2.0);
        assert_eq!(choices[0].y, (720.0 - total) / 2.0);
        assert_eq!(choices[1].y - choices[0].y, 56.0 + 16.0);

        let grid = config.choice_layout(|l| l.grid(2)).choice_rects(4, screen);
        assert_eq!(grid[0].y, grid[1].y);
        assert!(grid[1].x > grid[0].x);
    }
}

mod pause_and_save_menus {
    use vn_engine::raylib::prelude::Vector2;
    use vn_engine::{Action, PauseMenuConfig};

    #[test]
    fn a_grid_pause_menu_widens_its_panel() {
        let screen = Vector2::new(1280.0, 720.0);
        let column = PauseMenuConfig::default();
        let grid = PauseMenuConfig::default().layout(|l| l.grid(2));

        let (narrow, wide) = (column.panel(screen), grid.panel(screen));
        assert_eq!(narrow.width, 340.0);
        assert_eq!(wide.width, 2.0 * 260.0 + 10.0 + 2.0 * 28.0);
        assert!(wide.height < narrow.height);

        let one = PauseMenuConfig::default().button("Only", Action::Resume);
        assert_eq!(one.button_rects(screen).len(), 1);
    }
}

mod rows_of {
    use vn_engine::raylib::prelude::{Rectangle, Vector2};
    use vn_engine::{Action, Align, Anchor, Layout, MainMenuConfig};

    fn corners(rects: &[Rectangle]) -> Vec<(f32, f32)> {
        rects.iter().map(|r| (r.x, r.y)).collect()
    }

    #[test]
    fn one_then_two_columns_then_one() {
        let menu = [
            "New Game", "Continue", "Load", "Settings", "Credits", "Exit",
        ]
        .into_iter()
        .fold(
            MainMenuConfig::default().button_style(|b| b.size(260.0, 52.0)),
            |menu, label| menu.button(label, Action::Quit),
        )
        .layout(|l| l.rows_of([1, 2, 2, 1]).spacing_xy(24.0, 18.0));

        let rects = menu.button_rects(Vector2::new(1280.0, 720.0));
        let top = 720.0 * 0.45;
        let (left, right) = (640.0 - 260.0 - 12.0, 640.0 + 12.0);
        assert_eq!(
            corners(&rects),
            [
                (510.0, top),
                (left, top + 70.0),
                (right, top + 70.0),
                (left, top + 140.0),
                (right, top + 140.0),
                (510.0, top + 210.0),
            ]
        );
    }

    #[test]
    fn the_last_count_repeats() {
        let layout = Layout::default().rows_of([1, 3]);
        assert_eq!(layout.row_counts(8), [1, 3, 3, 1]);
        assert_eq!(layout.rows(8), 4);
        assert_eq!(layout.columns(8), 3);
        assert_eq!(Layout::default().grid(2).row_counts(5), [2, 2, 1]);
        assert_eq!(Layout::default().row().row_counts(3), [3]);
        assert_eq!(Layout::default().row_counts(0), Vec::<usize>::new());
    }

    #[test]
    fn empty_or_zero_patterns_are_a_column() {
        for layout in [
            Layout::default().rows_of([]),
            Layout::default().rows_of([0, 0]),
        ] {
            assert_eq!(layout.row_counts(3), [1, 1, 1]);
        }
        assert_eq!(Layout::default().rows_of([0, 2]).row_counts(4), [2, 2]);
    }

    #[test]
    fn rows_are_sized_and_aligned_on_their_own() {
        let sizes = [
            Vector2::new(400.0, 50.0),
            Vector2::new(100.0, 40.0),
            Vector2::new(100.0, 40.0),
        ];
        let area = Rectangle::new(0.0, 0.0, 1000.0, 600.0);
        let place = |align| {
            Layout::default()
                .rows_of([1, 2])
                .anchor(Anchor::TopLeft)
                .align(align)
                .spacing(10.0)
                .place(area, &sizes)
        };

        assert_eq!(
            corners(&place(Align::Center)),
            [(0.0, 0.0), (95.0, 60.0), (205.0, 60.0)]
        );
        assert_eq!(
            corners(&place(Align::Start)),
            [(0.0, 0.0), (0.0, 60.0), (110.0, 60.0)]
        );
        assert_eq!(
            corners(&place(Align::End)),
            [(0.0, 0.0), (190.0, 60.0), (300.0, 60.0)]
        );
        assert_eq!(
            Layout::default()
                .rows_of([1, 2])
                .spacing(10.0)
                .block_size(&sizes),
            Vector2::new(400.0, 100.0)
        );
    }

    #[test]
    fn the_real_size_is_used_to_fit_the_window() {
        let menu = (0..6).fold(
            MainMenuConfig::default().button_style(|b| b.size(260.0, 52.0)),
            |menu, i| menu.button(format!("{}", i), Action::Quit),
        );
        let screen = Vector2::new(1280.0, 720.0);

        let column_top = menu.button_rects(screen)[0].y;
        let rows_top = menu
            .layout(|l| l.rows_of([1, 2, 2, 1]))
            .button_rects(screen)[0]
            .y;
        assert!(column_top < 720.0 * 0.45, "six stacked buttons move up");
        assert_eq!(rows_top, 720.0 * 0.45, "four rows fit where they are");
    }
}

mod stretch {
    use vn_engine::raylib::prelude::{Rectangle, Vector2};
    use vn_engine::{Align, Anchor, Layout};

    fn place(layout: Layout, sizes: &[Vector2]) -> Vec<(f32, f32)> {
        layout
            .anchor(Anchor::TopLeft)
            .align(Align::Stretch)
            .spacing(24.0)
            .place(Rectangle::new(0.0, 0.0, 1000.0, 600.0), sizes)
            .iter()
            .map(|r| (r.x, r.width))
            .collect()
    }

    #[test]
    fn short_rows_fill_the_block_width() {
        let sizes = vec![Vector2::new(260.0, 52.0); 6];
        assert_eq!(
            place(Layout::default().rows_of([1, 2, 2, 1]), &sizes),
            [
                (0.0, 544.0),
                (0.0, 260.0),
                (284.0, 260.0),
                (0.0, 260.0),
                (284.0, 260.0),
                (0.0, 544.0),
            ]
        );
    }

    #[test]
    fn uneven_rows_share_the_spare_width() {
        let sizes = [
            Vector2::new(100.0, 40.0),
            Vector2::new(100.0, 40.0),
            Vector2::new(100.0, 40.0),
            Vector2::new(200.0, 40.0),
            Vector2::new(100.0, 40.0),
        ];
        assert_eq!(
            place(Layout::default().rows_of([3, 2]), &sizes),
            [
                (0.0, 100.0),
                (124.0, 100.0),
                (248.0, 100.0),
                (0.0, 212.0),
                (236.0, 112.0),
            ]
        );
    }

    #[test]
    fn column_items_fill_the_widest() {
        let sizes = [Vector2::new(300.0, 50.0), Vector2::new(100.0, 50.0)];
        assert_eq!(
            place(Layout::default(), &sizes),
            [(0.0, 300.0), (0.0, 300.0)]
        );
    }
}

mod character_positions {
    use vn_engine::PlayingConfig;
    use vn_engine::script::Position;

    #[test]
    fn five_spots_across_the_window() {
        let config = PlayingConfig::default();
        let xs = Position::ALL.map(|p| config.position_x(p));
        assert_eq!(xs, [0.15, 0.3, 0.5, 0.7, 0.85]);
        assert_eq!(config.character_height, Some(0.8));

        let moved = config.position(Position::Left, 0.25).character_height(None);
        assert_eq!(moved.position_x(Position::Left), 0.25);
        assert_eq!(moved.character_height, None);
    }
}
