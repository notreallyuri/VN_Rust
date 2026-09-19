use vn_script::{
    CharacterDef, Schema, Severity, StoryVm, closest, compile_source, did_you_mean, edit_distance,
};

fn first_error(source: &str) -> String {
    compile_source(source)
        .diagnostics
        .into_iter()
        .find(|d| d.severity == Severity::Error)
        .map(|d| d.message)
        .expect("an error")
}

#[test]
fn edit_distance_counts_insertions_deletions_substitutions_and_swaps() {
    assert_eq!(edit_distance("mary", "mary"), 0);
    assert_eq!(edit_distance("remoe", "remove"), 1);
    assert_eq!(edit_distance("marry", "mary"), 1);
    assert_eq!(edit_distance("shwo", "show"), 1);
    assert_eq!(edit_distance("hugo", "hugh"), 1);
    assert_eq!(edit_distance("", "set"), 3);
    assert_eq!(edit_distance("kitten", "sitting"), 3);
}

#[test]
fn closest_picks_the_nearest_candidate_within_a_limit() {
    let names = ["mary", "moriarty", "hugo"];
    assert_eq!(closest("marry", names), Some("mary"));
    assert_eq!(closest("moriraty", names), Some("moriarty"));
    assert_eq!(closest("adelaide", names), None);
    assert_eq!(closest("mary", names), None);
    assert_eq!(closest("x", ["y", "xy"]), Some("xy"));
}

#[test]
fn did_you_mean_formats_the_hint() {
    assert_eq!(did_you_mean("marry", ["mary"]), "; did you mean 'mary'?");
    assert_eq!(did_you_mean("zzz", ["mary"]), "");
}

#[test]
fn misspelled_keywords_are_suggested() {
    assert_eq!(
        first_error("scene start:\n  remoe hugo\n"),
        "expected dialogue (`<character> \"<text>\"`) or a keyword, found `remoe hugo`; did you mean `remove`?"
    );
    assert!(first_error("scene start:\n  shwo mary happy\n").ends_with("did you mean `show`?"));
    assert!(
        first_error("scene start:\n  chioce:\n    \"a\":\n      \"b\"\n")
            .ends_with("did you mean `choice`?")
    );
    assert!(first_error("scene start:\n  shw mary \"Hello\"\n").ends_with("did you mean `show`?"));
}

#[test]
fn unrelated_words_get_no_keyword_hint() {
    let message = first_error("scene start:\n  hello there\n");
    assert!(!message.contains("did you mean"), "{}", message);
}

fn vm(source: &str) -> StoryVm {
    let mut schema = Schema::default();
    schema.characters.insert(
        "mary".into(),
        CharacterDef {
            name: "Mary".into(),
            images: vec!["neutral".into(), "happy".into()],
        },
    );
    let mut vm = StoryVm::from_source(source);
    vm.set_schema(schema);
    vm
}

#[test]
fn unknown_scenes_and_images_are_suggested() {
    let messages: Vec<String> = vm("scene start:\n  show mary hapy\n  jump strat\n")
        .validate()
        .into_iter()
        .map(|d| d.message)
        .collect();
    assert_eq!(
        messages,
        [
            "`show mary hapy`: 'mary' has no image 'hapy' (images: neutral, happy); did you mean 'happy'?",
            "`jump strat`: no scene with that name; did you mean 'start'?",
        ]
    );
}

#[test]
fn a_missing_entry_scene_is_suggested() {
    let mut story = vm("scene prologue:\n  \"Hi\"\n");
    let diagnostics = story.prepare(Schema::default(), Some("prolog"));
    assert_eq!(
        diagnostics[0].message,
        "entry scene 'prolog' does not exist; did you mean 'prologue'?"
    );
}
