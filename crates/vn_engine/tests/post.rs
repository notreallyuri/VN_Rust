use vn_engine::frame::post::PostChain;

fn chain() -> PostChain {
    let mut chain = PostChain::new();
    chain.declare("grain", 1.0);
    chain.declare("desaturate", 0.5);
    chain
}

#[test]
fn passes_start_switched_off() {
    let chain = chain();
    assert_eq!(chain.active(), Vec::<&str>::new());
    assert!(!chain.busy());
    assert_eq!(chain.passes().len(), 2);
}

#[test]
fn passes_keep_the_order_they_were_added_in() {
    let mut chain = chain();
    chain.set_enabled("desaturate", true);
    chain.set_enabled("grain", true);
    assert_eq!(chain.active(), ["grain", "desaturate"]);
}

#[test]
fn a_pass_with_no_amount_does_nothing() {
    let mut chain = chain();
    chain.set_enabled("grain", true);
    chain.set_amount("grain", 0.0);
    assert_eq!(chain.active(), Vec::<&str>::new());

    chain.set_amount("grain", 0.3);
    assert_eq!(chain.active(), ["grain"]);
    assert_eq!(chain.passes()[0].amount(), 0.3);
}

#[test]
fn an_amount_is_never_negative() {
    let mut chain = chain();
    chain.set_amount("grain", -2.0);
    assert_eq!(chain.passes()[0].amount(), 0.0);
}

#[test]
fn an_unknown_name_is_reported_back() {
    let mut chain = chain();
    assert!(!chain.set_enabled("nope", true));
    assert!(!chain.set_amount("nope", 1.0));
    assert!(chain.set_enabled("grain", true));
}

#[test]
fn clearing_switches_everything_off() {
    let mut chain = chain();
    chain.set_enabled("grain", true);
    chain.set_enabled("desaturate", true);
    chain.clear();
    assert_eq!(chain.active(), Vec::<&str>::new());
}

#[test]
#[ignore = "opens a window; run with --ignored on a machine with a display"]
fn the_built_in_shaders_compile() {
    use raylib::prelude::*;

    let (mut rl, thread) = raylib::init().size(64, 64).title("shaders").build();
    rl.set_trace_log(TraceLogLevel::LOG_ERROR);

    let mut chain = PostChain::new();
    for (name, source) in [
        ("grain", vn_engine::frame::post::GRAIN),
        ("desaturate", vn_engine::frame::post::DESATURATE),
        ("blur", vn_engine::frame::post::BLUR),
        ("fxaa", vn_engine::frame::post::FXAA),
    ] {
        chain.load(&mut rl, &thread, name, source, 1.0);
        chain.set_enabled(name, true);
    }

    assert_eq!(chain.active().len(), 4);
    assert!(
        chain.busy(),
        "a built-in shader failed to compile, so it was dropped"
    );
}
