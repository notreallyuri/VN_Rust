use novn::action::Action;
use novn::context::GameView;
use novn::data::saves::Saves;
use novn::data::settings::Settings;
use novn::data::state::GameState;
use novn::raylib::prelude::{Color, Rectangle, Vector2};
use novn::screens::main_menu::{MenuItem, can_continue};
use novn::screens::playing::{HudButton, PlayingConfig};
use novn::script::StoryVm;
use novn::ui::button::{ButtonIcon, ButtonImage, ButtonStyle, Transform};
use novn::ui::button::{StateAmounts, mix_color, step_amount};

const RECT: Rectangle = Rectangle {
    x: 100.0,
    y: 50.0,
    width: 200.0,
    height: 40.0,
};

fn close(a: Vector2, b: Vector2) -> bool {
    (a.x - b.x).abs() < 1e-3 && (a.y - b.y).abs() < 1e-3
}

#[test]
fn transforms_round_trip_and_hit_test_what_is_drawn() {
    let transform = Transform::IDENTITY
        .scale_xy(1.2, 0.9)
        .rotate(17.0)
        .skew(-12.0, 4.0)
        .offset(6.0, -3.0);
    for point in [
        Vector2::new(100.0, 50.0),
        Vector2::new(250.0, 70.0),
        Vector2::new(300.0, 90.0),
    ] {
        let moved = transform.apply(RECT, point);
        assert!(close(transform.invert(RECT, moved).unwrap(), point));
    }

    let turned = Transform::IDENTITY.rotate(90.0);
    let center = Vector2::new(200.0, 70.0);
    assert!(
        turned.contains(RECT, Vector2::new(200.0, 150.0)),
        "now tall"
    );
    assert!(
        !turned.contains(RECT, Vector2::new(290.0, 70.0)),
        "no longer wide"
    );
    assert!(turned.contains(RECT, center));

    let slanted = Transform::IDENTITY.skew(30.0, 0.0);
    let top_left = slanted.apply(RECT, Vector2::new(100.0, 50.0));
    assert!(top_left.x < 100.0, "the top leans left: {:?}", top_left);
    assert!(slanted.contains(RECT, top_left));

    assert!(Transform::IDENTITY.is_identity());
    assert!(Transform::IDENTITY.origin(0.0, 0.0).is_identity());
    assert!(!Transform::IDENTITY.scale(0.98).is_identity());
    assert!(
        Transform::IDENTITY
            .scale(0.0)
            .invert(RECT, center)
            .is_none()
    );
}

#[test]
fn states_blend_over_the_base_look() {
    let style = ButtonStyle::default()
        .color(Color::new(100, 0, 0, 255))
        .border(1.0, Color::new(0, 0, 0, 255))
        .hovered(|l| {
            l.fill(Color::new(200, 0, 0, 255))
                .border(3.0, Color::new(255, 255, 255, 255))
                .transform(|t| t.offset(10.0, 0.0))
        })
        .pressed(|l| l.transform(|t| t.scale(0.9)));

    let normal = style.look(StateAmounts::default());
    assert_eq!(normal.fill, Color::new(100, 0, 0, 255));
    assert_eq!(normal.border.unwrap().width, 1.0);

    let halfway = style.look(StateAmounts {
        hover: 0.5,
        ..StateAmounts::default()
    });
    assert_eq!(halfway.fill, Color::new(150, 0, 0, 255));
    assert_eq!(halfway.border.unwrap().width, 2.0);
    assert_eq!(halfway.transform.offset.x, 5.0);

    let pressed = style.look(StateAmounts {
        hover: 1.0,
        press: 1.0,
        ..StateAmounts::default()
    });
    assert_eq!(
        pressed.fill,
        Color::new(100, 0, 0, 255),
        "`color` sets the pressed fill"
    );
    assert_eq!(pressed.transform.scale.x, 0.9);

    let disabled = style.look(StateAmounts {
        disabled: true,
        ..StateAmounts::default()
    });
    assert_eq!(disabled.opacity, 0.45);

    let focused = ButtonStyle::default().look(StateAmounts {
        focus: 1.0,
        ..StateAmounts::default()
    });
    assert_eq!(
        focused.border.unwrap().width,
        2.0,
        "focus adds a border by default"
    );
}

#[test]
fn images_switch_halfway_and_every_path_is_listed() {
    let style = ButtonStyle::default()
        .image(ButtonImage::new("ui/button.png").slice_all(12))
        .hovered(|l| l.image(ButtonImage::new("ui/button_hover.png")))
        .icon(ButtonIcon::new("ui/save.png"));

    let at = |hover| {
        style
            .look(StateAmounts {
                hover,
                ..StateAmounts::default()
            })
            .image
            .unwrap()
            .path
    };
    assert_eq!(at(0.4), "ui/button.png");
    assert_eq!(at(0.5), "ui/button_hover.png");
    assert_eq!(
        style.image_paths(),
        ["ui/button.png", "ui/button_hover.png", "ui/save.png"]
    );
    assert_eq!(
        style.image.as_ref().unwrap().slice.unwrap().left,
        12,
        "nine-slice borders"
    );
}

#[test]
fn transitions_step_towards_their_target() {
    assert_eq!(step_amount(0.0, 1.0, 0.05, 0.2), 0.25);
    assert_eq!(step_amount(0.9, 1.0, 0.05, 0.2), 1.0);
    assert_eq!(step_amount(1.0, 0.0, 0.1, 0.2), 0.5);
    assert_eq!(step_amount(0.3, 1.0, 0.01, 0.0), 1.0, "no transition snaps");
    assert_eq!(
        mix_color(Color::new(0, 0, 0, 0), Color::new(255, 100, 50, 200), 0.5),
        Color::new(128, 50, 25, 100)
    );
}

#[test]
fn hud_and_choice_buttons_can_each_have_a_style() {
    let config = PlayingConfig::default()
        .hud_item(HudButton::new("Save", Action::QuickSave))
        .hud_item(
            HudButton::new("Menu", Action::Resume).style(|b| b.size(90.0, 30.0).skew(-10.0, 0.0)),
        )
        .choice_button_for(|index, text, b| {
            if text.starts_with('[') {
                b.color(Color::new(90, 20, 20, 255))
            } else {
                b.font_size(18.0 + index as f32)
            }
        });

    assert_eq!(config.hud_style(0), config.hud_button);
    let menu = config.hud_style(1);
    assert_eq!((menu.width, menu.height), (90.0, 30.0));
    assert_eq!(menu.transform.skew.x, -10.0);
    let rects = config.hud_rects(Vector2::new(1280.0, 720.0));
    assert_eq!(rects[1].width, 90.0, "layout uses each button's own size");

    assert_eq!(
        config.choice_style_for(0, "[Lie]").color,
        Color::new(90, 20, 20, 255)
    );
    assert_eq!(config.choice_style_for(2, "Tell the truth").text.size, 20.0);
}

#[test]
fn menu_items_can_be_disabled() {
    let dir = std::env::temp_dir().join(format!("vn_engine_button_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let saves = Saves::new(&dir, "Test");
    let mut story = StoryVm::from_source("scene a:\n  \"hi\"\n");
    let state = GameState::default();
    let settings = Settings::default();
    let persistent = novn::data::persistent::Persistent::in_memory();

    let view = |story: &StoryVm| {
        let view = GameView {
            story,
            state: &state,
            persistent: &persistent,
            saves: &saves,
            settings: &settings,
        };
        (
            can_continue(&view),
            MenuItem::new("Gallery", Action::Resume)
                .enabled_if(|v| v.settings.text_speed > 100)
                .is_enabled(&view),
            MenuItem::new("New", Action::NewGame).is_enabled(&view),
        )
    };

    assert_eq!(view(&story), (false, false, true));
    story.advance_until_blocking();
    assert!(view(&story).0, "a story in progress can continue");
}
