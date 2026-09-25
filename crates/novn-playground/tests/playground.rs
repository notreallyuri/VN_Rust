use novn_playground::{Request, run, run_json};

const STORY: &str = r#"scene start:
  show mary tired
  mary "Your hot water, miss."
  choice:
    "Thank her":
      jump after
    "Say nothing":
      "The silence holds."
  "Unreachable once an option jumps."

scene after:
  "She smiles."
"#;

fn ask(source: &str, picks: &[usize]) -> novn_playground::Response {
    run(Request {
        source: source.to_string(),
        picks: picks.to_vec(),
    })
}

#[test]
fn a_story_compiles_and_stops_at_its_choice() {
    let response = ask(STORY, &[]);
    assert!(response.ok);
    assert!(response.notes.is_empty(), "{:?}", response.notes.len());
    assert_eq!(response.counts.scenes, 2);
    assert_eq!(
        response
            .steps
            .iter()
            .map(|step| step.kind)
            .collect::<Vec<_>>(),
        ["show", "dialogue"]
    );
    assert_eq!(response.choices.len(), 2);
    assert!(!response.ended);
}

#[test]
fn a_pick_replays_the_story_through_it() {
    let response = ask(STORY, &[0]);
    assert!(response.ended);
    assert!(response.choices.is_empty());
    let taken: Vec<_> = response
        .steps
        .iter()
        .filter_map(|step| step.taken.as_deref())
        .collect();
    assert_eq!(taken, ["Thank her"]);
    assert!(
        response
            .steps
            .iter()
            .any(|step| step.text.as_deref() == Some("She smiles."))
    );
}

#[test]
fn a_gated_option_says_why_it_is_closed() {
    let source = "scene start:\n  choice:\n    \"Open\":\n      \"yes\"\n    \"Shut\" when trust >= 3 \"You do not trust her\":\n      \"no\"\n";
    let response = ask(source, &[]);
    let shut = response
        .choices
        .iter()
        .find(|option| option.text == "Shut")
        .expect("the gated option is listed");
    assert!(!shut.enabled);
    assert_eq!(shut.reason.as_deref(), Some("You do not trust her"));
}

#[test]
fn errors_carry_their_line_and_suggestion() {
    let response = ask("scene start:\n  remoe hugo\n", &[]);
    assert!(!response.ok);
    let note = &response.notes[0];
    assert_eq!(note.severity, "error");
    assert_eq!(note.line, 2);
    assert!(note.message.contains("did you mean `remove`?"), "{note:?}");
}

#[test]
fn a_jump_out_of_the_snippet_is_a_warning_not_an_error() {
    let response = ask("scene start:\n  \"one line\"\n  jump nowhere\n", &[]);
    assert!(
        response.ok,
        "a snippet that jumps somewhere it does not define is still worth running"
    );
    let note = response
        .notes
        .iter()
        .find(|note| note.message.contains("'nowhere'"))
        .expect("the jump is reported");
    assert_eq!(note.severity, "warning");
    assert!(note.message.contains("does not define"), "{note:?}");
    assert!(
        response
            .steps
            .iter()
            .any(|step| step.text.as_deref() == Some("one line")),
        "the lines before the jump still play"
    );
}

#[test]
fn a_fragment_is_wrapped_in_a_scene_so_it_can_run() {
    let response = ask("mary \"Hello.\"\n\"She waits.\"\n", &[]);
    assert!(response.ok, "{:?}", response.notes);
    assert!(response.wrapped);
    assert_eq!(
        response
            .steps
            .iter()
            .filter_map(|s| s.text.as_deref())
            .collect::<Vec<_>>(),
        ["Hello.", "She waits."]
    );
}

#[test]
fn a_story_that_declares_its_own_scene_is_not_wrapped() {
    let response = ask("scene start:\n  \"x\"\n", &[]);
    assert!(!response.wrapped);
    assert!(response.listing.contains("scene start:"));
}

#[test]
fn an_elision_becomes_narration_rather_than_an_error() {
    let response = ask(
        "choice final:\n  \"Keep the letter\":\n    ...\n  \"Burn the letter\":\n    ...\n",
        &[],
    );
    assert!(response.ok, "{:?}", response.notes);
    assert_eq!(response.choices.len(), 2);
    let after = ask(
        "choice final:\n  \"Keep the letter\":\n    ...\n  \"Burn the letter\":\n    ...\n",
        &[0],
    );
    assert!(
        after
            .steps
            .iter()
            .any(|step| step.text.as_deref() == Some("...")),
        "the elision plays as a narration line"
    );
}

#[test]
fn an_elision_inside_a_string_is_left_alone() {
    let response = ask("mary \"Well...\"\n", &[]);
    assert!(response.ok, "{:?}", response.notes);
    assert_eq!(response.steps[0].text.as_deref(), Some("Well..."));
}

#[test]
fn an_indented_fragment_keeps_its_own_nesting() {
    let response = ask(
        "choice:\n  \"Yes\":\n    \"You agree.\"\n  \"No\":\n    \"You refuse.\"\n",
        &[],
    );
    assert!(response.ok, "{:?}", response.notes);
    assert!(response.wrapped);
    assert_eq!(response.choices.len(), 2);
}

#[test]
fn the_listing_is_the_one_novn_dump_prints() {
    let response = ask(STORY, &[]);
    assert!(response.listing.contains("scene start:"));
    assert!(response.listing.contains("SHOW mary tired"));
    assert!(response.listing.contains("CHOICE"));
}

#[test]
fn a_loop_stops_instead_of_running_forever() {
    let response = ask("scene a:\n  jump a\n", &[]);
    assert!(response.ok);
    assert!(response.ended || response.stopped.is_some());
}

#[test]
fn a_pick_that_is_not_on_offer_is_reported_not_panicked() {
    let response = ask(STORY, &[99]);
    assert!(response.stopped.is_some(), "{:?}", response.stopped);
}

#[test]
fn a_gated_option_cannot_be_taken() {
    let source = "scene start:\n  choice:\n    \"Open\":\n      \"yes\"\n    \"Shut\" when trust >= 3 \"You do not trust her\":\n      \"no\"\n";
    let response = ask(source, &[1]);
    assert!(
        response.stopped.is_some(),
        "the option is shown but disabled, so choosing it cannot succeed"
    );
}

#[test]
fn an_option_hidden_by_a_gate_is_not_offered() {
    let source = "scene start:\n  choice:\n    \"Open\":\n      \"yes\"\n    \"Hidden\" when trust >= 3:\n      \"no\"\n";
    let response = ask(source, &[]);
    assert_eq!(
        response.choices.len(),
        1,
        "a gate with no reason hides the option rather than disabling it"
    );
    assert_eq!(response.choices[0].text, "Open");
}

#[test]
fn nothing_in_the_json_path_panics_on_rubbish() {
    for request in [
        "",
        "{}",
        "not json",
        r#"{"source":123}"#,
        r#"{"picks":[1]}"#,
        r#"{"source":"\u0000","picks":[0,0,0]}"#,
    ] {
        let answer = run_json(request);
        assert!(answer.starts_with('{'), "{request}: {answer}");
        assert!(answer.contains("\"ok\""), "{request}: {answer}");
    }
}

#[test]
fn an_empty_story_is_not_an_error() {
    let response = ask("", &[]);
    assert_eq!(response.counts.instructions, 0);
    assert!(response.steps.is_empty());
}
