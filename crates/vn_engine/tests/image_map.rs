use raylib::prelude::*;
use vn_engine::{Action, GameState, GameView, Hotspot, ImageMap, ImageMapStyle, Settings, Shape};
use vn_engine::{Saves, script::StoryVm};

fn area() -> Rectangle {
    Rectangle::new(0.0, 0.0, 1280.0, 720.0)
}

fn room() -> ImageMap {
    ImageMap::new()
        .background("backgrounds/study.png")
        .hotspot(Hotspot::new("room", Shape::all()).label("The study"))
        .hotspot(
            Hotspot::new(
                "desk",
                Shape::rect(320.0, 360.0, 320.0, 180.0).in_image(1280.0, 720.0),
            )
            .label("The desk")
            .tooltip("Papers, and a locked drawer"),
        )
        .hotspot(Hotspot::new(
            "lamp",
            Shape::circle(960.0, 300.0, 64.0).in_image(1280.0, 720.0),
        ))
}

#[test]
fn hotspots_are_found_by_id_and_by_point() {
    let room = room();
    assert_eq!(room.find("desk"), Some(1));
    assert_eq!(room.find("window"), None);

    assert_eq!(room.hovered_at(area(), Vector2::new(400.0, 400.0)), Some(1));
    assert_eq!(room.hovered_at(area(), Vector2::new(960.0, 300.0)), Some(2));
    assert_eq!(
        room.hovered_at(area(), Vector2::new(100.0, 100.0)),
        Some(0),
        "a hotspot covering the picture still answers for the rest of it"
    );
    assert_eq!(room.hovered_at(area(), Vector2::new(-10.0, 100.0)), None);
}

#[test]
fn the_hotspot_drawn_last_takes_the_point() {
    let stacked = ImageMap::new()
        .hotspot(Hotspot::new("wall", Shape::rect(0.0, 0.0, 1.0, 1.0)))
        .hotspot(Hotspot::new("portrait", Shape::rect(0.4, 0.2, 0.2, 0.3)));

    assert_eq!(
        stacked.hovered_at(area(), Vector2::new(640.0, 200.0)),
        Some(1)
    );
    assert_eq!(
        stacked.hovered_at(area(), Vector2::new(640.0, 600.0)),
        Some(0)
    );
}

#[test]
fn hotspots_can_be_disabled_by_the_game_state() {
    let dir = std::env::temp_dir().join(format!("vn_engine_map_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let saves = Saves::new(&dir, "Test");
    let story = StoryVm::from_source("scene a:\n  \"hi\"\n");
    let state = GameState::default();
    let settings = Settings::default();
    let view = GameView {
        story: &story,
        state: &state,
        saves: &saves,
        settings: &settings,
    };

    let always = Hotspot::new("door", Shape::all());
    let never =
        Hotspot::new("drawer", Shape::all()).enabled_if(|view| view.story.current().is_some());

    assert!(always.is_enabled(&view));
    assert!(!never.is_enabled(&view));
}

#[test]
fn a_hotspot_carries_its_label_tooltip_and_action() {
    let room = room();
    let desk = &room.hotspots[1];

    assert_eq!(desk.label.as_deref(), Some("The desk"));
    assert_eq!(desk.tooltip.as_deref(), Some("Papers, and a locked drawer"));
    assert!(desk.action.is_none());

    let with_action = Hotspot::new("door", Shape::all()).action(Action::Resume);
    assert!(matches!(with_action.action, Some(Action::Resume)));
}

#[test]
fn hotspots_follow_the_picture_when_the_window_does_not_match_it() {
    let picture = Vector2::new(1280.0, 720.0);
    let desk = Shape::rect(320.0, 360.0, 320.0, 180.0).in_image(1280.0, 720.0);

    let same = vn_engine::ui::cover_rect(picture, area());
    assert_eq!(desk.bounds(same).x, 320.0);

    let wide = vn_engine::ui::cover_rect(picture, Rectangle::new(0.0, 0.0, 1600.0, 720.0));
    assert_eq!((wide.width, wide.height), (1600.0, 900.0));
    assert_eq!((wide.x, wide.y), (0.0, -90.0));
    assert_eq!(
        desk.bounds(wide).x,
        400.0,
        "the desk stays over the desk when the picture is cropped to fit"
    );
}

#[test]
fn the_default_style_only_shows_a_hotspot_under_the_pointer() {
    let style = ImageMapStyle::default();
    assert!(style.idle.is_empty());
    assert!(!style.hovered.is_empty());
    assert!(!style.focused.is_empty());
}
