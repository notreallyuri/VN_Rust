use novn::context::GameView;
use novn::data::saves::Saves;
use novn::data::settings::Settings;
use novn::data::state::GameState;
use novn::input::drag::{DragBoard, DragStyle, Draggable, DropTarget};
use novn::input::hit::{Highlight, Shape};
use novn::script::StoryVm;
use raylib::prelude::*;

fn area() -> Rectangle {
    Rectangle::new(0.0, 0.0, 1000.0, 500.0)
}

fn board() -> DragBoard {
    DragBoard::new()
        .item(
            Draggable::new("ledger", Shape::rect(0.0, 0.0, 0.2, 0.2))
                .label("The ledger")
                .image("ui/ledger.png"),
        )
        .item(Draggable::new("key", Shape::rect(0.3, 0.0, 0.2, 0.2)))
        .target(DropTarget::new("shelf", Shape::rect(0.0, 0.6, 0.4, 0.4)).label("The shelf"))
        .target(DropTarget::new("drawer", Shape::rect(0.5, 0.6, 0.4, 0.4)))
}

fn always(_item: usize, _target: usize) -> bool {
    true
}

#[test]
fn items_and_targets_are_found_under_a_point() {
    let board = board();

    assert_eq!(board.item_at(area(), Vector2::new(50.0, 50.0)), Some(0));
    assert_eq!(board.item_at(area(), Vector2::new(350.0, 50.0)), Some(1));
    assert_eq!(board.item_at(area(), Vector2::new(350.0, 400.0)), None);

    assert_eq!(board.target_at(area(), Vector2::new(100.0, 400.0)), Some(0));
    assert_eq!(board.target_at(area(), Vector2::new(600.0, 400.0)), Some(1));
    assert_eq!(board.target_at(area(), Vector2::new(450.0, 400.0)), None);
}

#[test]
fn a_held_item_follows_the_pointer() {
    let mut board = board();
    let rect = board.item_rect(0, area());
    assert_eq!((rect.x, rect.y), (0.0, 0.0));

    board.grab(0, Vector2::new(60.0, 40.0), false);
    assert_eq!(board.held(), Some(0));
    assert_eq!(board.held_id(), Some("ledger"));

    board.drag(Vector2::new(260.0, 140.0));
    assert_eq!(board.offset(0), Vector2::new(200.0, 100.0));
    assert_eq!(board.offset(1), Vector2::zero());

    let rect = board.item_rect(0, area());
    assert_eq!((rect.x, rect.y), (200.0, 100.0));
    assert_eq!((rect.width, rect.height), (200.0, 100.0));
    assert_eq!(board.item_rect(1, area()).x, 300.0);
}

#[test]
fn a_drop_names_the_target_it_landed_on() {
    let mut board = board();
    board.grab(1, Vector2::new(350.0, 50.0), false);
    board.drag(Vector2::new(600.0, 400.0));

    let drop = board
        .release(area(), Vector2::new(600.0, 400.0), always)
        .unwrap();
    assert_eq!(drop.item_id, "key");
    assert_eq!(drop.target, Some(1));
    assert_eq!(drop.target_id.as_deref(), Some("drawer"));
    assert!(drop.accepted);
    assert_eq!(board.held(), None);
    assert_eq!(board.offset(1), Vector2::zero());
}

#[test]
fn a_drop_on_nothing_is_not_accepted() {
    let mut board = board();
    board.grab(0, Vector2::new(50.0, 50.0), false);

    let drop = board
        .release(area(), Vector2::new(450.0, 400.0), always)
        .unwrap();
    assert_eq!(drop.target, None);
    assert_eq!(drop.target_id, None);
    assert!(!drop.accepted);
}

#[test]
fn a_target_that_refuses_the_item_reports_the_refusal() {
    let mut board = board();
    board.grab(0, Vector2::new(50.0, 50.0), false);

    let drop = board
        .release(area(), Vector2::new(100.0, 400.0), |_, target| target != 0)
        .unwrap();
    assert_eq!(drop.target_id.as_deref(), Some("shelf"));
    assert!(
        !drop.accepted,
        "the drop still reports where it landed so the game can say why"
    );
}

#[test]
fn cancelling_puts_the_item_back() {
    let mut board = board();
    board.grab(0, Vector2::new(50.0, 50.0), false);
    board.drag(Vector2::new(400.0, 400.0));

    let drop = board.cancel().unwrap();
    assert_eq!(drop.item_id, "ledger");
    assert_eq!(drop.target, None);
    assert!(!drop.accepted);
    assert_eq!(board.held(), None);
    assert_eq!(board.item_rect(0, area()).x, 0.0);
    assert!(board.cancel().is_none());
}

#[test]
fn a_validity_rule_reads_the_item_and_the_game() {
    let dir = std::env::temp_dir().join(format!("vn_engine_drag_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let saves = Saves::new(&dir, "Test");
    let story = StoryVm::from_source("scene a:\n  \"hi\"\n");
    let state = GameState::default();
    let settings = Settings::default();
    let persistent = novn::data::persistent::Persistent::in_memory();
    let view = GameView {
        story: &story,
        state: &state,
        persistent: &persistent,
        saves: &saves,
        settings: &settings,
    };

    let anything = DropTarget::new("bin", Shape::all());
    let keys_only = DropTarget::new("lock", Shape::all()).accepts(|item, _| item == "key");
    let started = DropTarget::new("case", Shape::all())
        .accepts(|_, view: &GameView| view.story.current().is_some());

    assert!(anything.allows("ledger", &view));
    assert!(keys_only.allows("key", &view));
    assert!(!keys_only.allows("ledger", &view));
    assert!(!started.allows("key", &view));
}

#[test]
fn replacing_the_items_drops_what_was_held() {
    let mut board = board();
    board.grab(1, Vector2::new(350.0, 50.0), false);

    board.set_items([Draggable::new("key", Shape::rect(0.3, 0.0, 0.2, 0.2))]);
    assert_eq!(board.held(), None, "the held item no longer exists");

    board.grab(0, Vector2::new(350.0, 50.0), false);
    board.set_items([
        Draggable::new("key", Shape::rect(0.3, 0.0, 0.2, 0.2)),
        Draggable::new("seal", Shape::rect(0.6, 0.0, 0.2, 0.2)),
    ]);
    assert_eq!(
        board.held(),
        Some(0),
        "an item that is still there stays held"
    );
}

#[test]
fn grabbing_an_item_that_is_not_there_holds_nothing() {
    let mut board = board();
    board.grab(7, Vector2::new(50.0, 50.0), false);
    assert_eq!(board.held(), None);
    assert!(
        board
            .release(area(), Vector2::new(100.0, 400.0), always)
            .is_none()
    );
}

#[test]
fn the_default_style_marks_targets_a_held_item_can_go_to() {
    let style = DragStyle::default();
    assert!(!style.target_ready.is_empty());
    assert!(!style.target_blocked.is_empty());
    assert_ne!(style.target_ready, style.target_blocked);

    let board = board().style(|s| s.item(Highlight::new().fill(Color::RED)));
    assert_eq!(board.style.item.fill, Some(Color::RED));
}
