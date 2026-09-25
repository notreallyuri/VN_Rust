use novn::overlay::OverlayRequest;
use novn::request::{Request, Requests};
use novn::ui::toast::Toast;

fn asked(requests: impl IntoIterator<Item = Request>) -> novn::request::Resolved {
    let mut list = Requests::default();
    for request in requests {
        list.push(request);
    }
    let resolved = list.resolve();
    assert!(list.is_empty(), "resolving consumes what was asked");
    resolved
}

#[test]
fn overlays_are_applied_in_the_order_they_were_asked_for() {
    let resolved = asked([
        Request::Overlay(OverlayRequest::Open("pause".into())),
        Request::Overlay(OverlayRequest::Close),
        Request::Overlay(OverlayRequest::Open("log".into())),
    ]);
    assert_eq!(
        resolved.overlays,
        [
            OverlayRequest::Open("pause".into()),
            OverlayRequest::Close,
            OverlayRequest::Open("log".into()),
        ],
        "opening then closing is not the same as closing then opening"
    );
}

#[test]
fn the_last_tooltip_and_toast_of_a_frame_win() {
    let resolved = asked([
        Request::Tooltip("behind".into()),
        Request::Toast(Toast::info("first")),
        Request::Tooltip("in front".into()),
        Request::Toast(Toast::error("second")),
    ]);
    assert_eq!(resolved.tooltip.as_deref(), Some("in front"));
    assert_eq!(
        resolved.toast.map(|t| t.text),
        Some("second".to_string()),
        "two notifications in one frame show the newer one"
    );
}

#[test]
fn asking_twice_in_a_frame_still_does_the_work_once() {
    let resolved = asked([
        Request::Autosave,
        Request::Screenshot,
        Request::Autosave,
        Request::Screenshot,
    ]);
    assert!(resolved.autosave);
    assert!(resolved.screenshot);
}

#[test]
fn a_quiet_frame_asks_for_nothing() {
    let resolved = asked([]);
    assert_eq!(resolved, novn::request::Resolved::default());
    assert!(resolved.overlays.is_empty());
    assert!(!resolved.autosave && !resolved.screenshot);
    assert!(resolved.tooltip.is_none() && resolved.toast.is_none());
}

#[test]
fn what_a_screen_asked_for_can_be_read_back_before_it_is_applied() {
    let mut list = Requests::default();
    list.push(Request::Autosave);
    list.push(Request::Tooltip("the ledger".into()));
    assert_eq!(list.len(), 2);
    assert!(
        list.iter().any(|r| matches!(r, Request::Autosave)),
        "a test can assert a screen's intent without opening a window"
    );
}

#[cfg(feature = "character-visuals")]
#[test]
fn a_screen_asks_for_the_characters_its_backend_should_draw() {
    use novn::game::visuals::VisualKey;

    let resolved = asked([
        Request::Visual(VisualKey::new("mary", "happy")),
        Request::Visual(VisualKey::new("hugo", "neutral")),
        Request::Visual(VisualKey::new("mary", "neutral")),
    ]);
    assert_eq!(
        resolved.visuals,
        [
            VisualKey::new("mary", "happy"),
            VisualKey::new("hugo", "neutral"),
            VisualKey::new("mary", "neutral"),
        ],
        "every appearance asked for, in order: one character can be mid-change"
    );

    assert!(
        asked([Request::Screenshot]).visuals.is_empty(),
        "a frame nobody asked in leaves the instances alone"
    );
}
