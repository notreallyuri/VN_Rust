use vn_script::translate::{self, Catalog, Entry, Source, StringKind};
use vn_script::{CharacterDef, Event, Schema, StoryVm, compile_sources};

const STORY: &str = r#"scene start:
  "The lamps are never put out down here."
  mary "Your hot water, miss."
  choice:
    "Open the letter":
      set read = true
    "Leave it sealed":
      "You put it back."
"#;

fn program(source: &str) -> vn_script::Program {
    compile_sources([("story/01_box.story", source)])
}

fn strings(source: &str) -> Vec<Source> {
    translate::extract(&program(source))
}

fn schema() -> Schema {
    let mut schema = Schema::default();
    schema
        .variables
        .insert("read".into(), vn_script::VariableDef::bool(false));
    schema.characters.insert(
        "mary".into(),
        CharacterDef {
            name: "Mary".into(),
            images: vec!["neutral".into()],
        },
    );
    schema
}

#[test]
fn every_line_a_player_reads_is_extracted() {
    let strings = strings(STORY);
    let kinds: Vec<StringKind> = strings.iter().map(|s| s.kind).collect();

    assert_eq!(
        kinds,
        [
            StringKind::Narration,
            StringKind::Dialogue,
            StringKind::Choice,
            StringKind::Choice,
            StringKind::Narration,
        ]
    );
    assert_eq!(strings[1].speaker.as_deref(), Some("mary"));
    assert_eq!(strings[1].text, "Your hot water, miss.");
    assert_eq!(strings[2].text, "Open the letter");
    assert_eq!(strings[0].line, 2);
}

#[test]
fn a_string_is_keyed_by_its_file_and_its_text() {
    let here = translate::key("01_box.story", "Hello");
    assert_eq!(here, translate::key("01_box.story", "Hello"));
    assert_ne!(here, translate::key("02_box.story", "Hello"));
    assert_ne!(here, translate::key("01_box.story", "Hello."));
    assert_eq!(here.len(), 16);
}

#[test]
fn the_file_key_ignores_where_the_story_was_loaded_from() {
    assert_eq!(translate::file_key("assets/story/01.story"), "01.story");
    assert_eq!(translate::file_key("<embedded>/story/01.story"), "01.story");
    assert_eq!(
        translate::file_key("/home/someone/game/assets/story/01.story"),
        "01.story"
    );
    assert_eq!(translate::file_key("01.story"), "01.story");
}

#[test]
fn extracting_twice_adds_nothing_the_second_time() {
    let strings = strings(STORY);
    let names = translate::extract_names(&schema());
    let mut catalog = Catalog::new("pt-BR");

    let first = catalog.refresh(&strings, &names);
    assert_eq!(first.added, 5 + 1);
    assert_eq!(first.total, 6);
    assert_eq!(first.translated, 0);
    assert_eq!(first.missing(), 6);

    let second = catalog.refresh(&strings, &names);
    assert_eq!(second.added, 0);
    assert_eq!(second.total, 6);
}

#[test]
fn an_edited_line_goes_stale_and_the_rest_is_kept() {
    let names = translate::extract_names(&schema());
    let mut catalog = Catalog::new("pt-BR");
    catalog.refresh(&strings(STORY), &names);

    let file = catalog.story.get_mut("01_box.story").unwrap();
    for entry in file.values_mut() {
        entry.text = format!("<{}>", entry.source);
    }

    let edited = STORY.replace("Your hot water, miss.", "Your hot water, madam.");
    let refresh = catalog.refresh(&strings(&edited), &names);

    assert_eq!(refresh.added, 1, "the edited line is a new string");
    assert_eq!(refresh.stale, 1, "its old translation is kept but stale");

    let catalog_text = |source: &str| catalog.text("01_box.story", source).map(str::to_string);
    assert_eq!(
        catalog_text("The lamps are never put out down here."),
        Some("<The lamps are never put out down here.>".to_string()),
        "untouched lines keep their translation"
    );
    assert_eq!(catalog_text("Your hot water, madam."), None);
    assert_eq!(
        catalog_text("Your hot water, miss."),
        None,
        "the stale entry is not used any more"
    );
    assert_eq!(catalog.stale().len(), 1);
}

#[test]
fn a_renamed_character_goes_stale() {
    let mut catalog = Catalog::new("pt-BR");
    catalog.refresh(&[], &translate::extract_names(&schema()));
    catalog.names.get_mut("mary").unwrap().text = "Maria".into();
    assert_eq!(catalog.name("mary", "Mary"), Some("Maria"));

    let mut renamed = schema();
    renamed.characters.get_mut("mary").unwrap().name = "Miss Von Lucis".into();
    let refresh = catalog.refresh(&[], &translate::extract_names(&renamed));

    assert_eq!(refresh.stale, 1);
    assert_eq!(catalog.name("mary", "Miss Von Lucis"), None);
    assert_eq!(
        catalog.names.get("mary").unwrap().text,
        "Maria",
        "the old translation is still in the file to work from"
    );
}

#[test]
fn a_catalog_round_trips_through_json() {
    let mut catalog = Catalog::new("ja");
    catalog.refresh(&strings(STORY), &translate::extract_names(&schema()));
    catalog
        .story
        .get_mut("01_box.story")
        .unwrap()
        .values_mut()
        .next()
        .unwrap()
        .text = "ここは".into();

    let json = catalog.to_json();
    assert_eq!(Catalog::from_json(&json), Ok(catalog));
    assert!(json.contains("\"language\": \"ja\""));

    let newer = json.replace("\"format_version\": 1", "\"format_version\": 99");
    assert!(Catalog::from_json(&newer).is_err());
}

#[test]
fn an_empty_or_stale_entry_is_not_a_translation() {
    let mut catalog = Catalog::new("pt-BR");
    let entry = |text: &str, stale: bool| Entry {
        source: "Hello".into(),
        text: text.into(),
        stale,
        ..Entry::default()
    };
    let file = catalog.story.entry("01.story".into()).or_default();
    file.insert(translate::key("01.story", "Hello"), entry("Olá", false));
    assert_eq!(catalog.text("01.story", "Hello"), Some("Olá"));

    let file = catalog.story.get_mut("01.story").unwrap();
    file.insert(translate::key("01.story", "Hello"), entry("", false));
    assert_eq!(catalog.text("01.story", "Hello"), None);

    let file = catalog.story.get_mut("01.story").unwrap();
    file.insert(translate::key("01.story", "Hello"), entry("Olá", true));
    assert_eq!(catalog.text("01.story", "Hello"), None);
}

fn translated_vm() -> StoryVm {
    let mut catalog = Catalog::new("pt-BR");
    catalog.refresh(&strings(STORY), &[]);
    let file = catalog.story.get_mut("01_box.story").unwrap();
    for entry in file.values_mut() {
        if entry.source.starts_with("Your hot water") {
            entry.text = "Sua água quente, senhorita.".into();
        }
        if entry.source == "Open the letter" {
            entry.text = "Abrir a carta".into();
        }
    }

    let mut vm = StoryVm::from_program(program(STORY));
    vm.set_catalog(Some(catalog));
    vm
}

#[test]
fn the_vm_reads_lines_out_of_the_catalog() {
    let mut vm = translated_vm();
    assert_eq!(vm.language(), Some("pt-BR"));

    assert_eq!(
        vm.advance(),
        Event::Say {
            speaker: None,
            text: "The lamps are never put out down here.".into(),
        },
        "a line with no translation falls back to the source"
    );
    assert_eq!(
        vm.advance(),
        Event::Say {
            speaker: Some("mary".into()),
            text: "Sua água quente, senhorita.".into(),
        }
    );
    assert_eq!(
        vm.advance(),
        Event::Choice {
            options: vec!["Abrir a carta".into(), "Leave it sealed".into()],
        }
    );
}

#[test]
fn a_translated_line_still_gets_its_variables() {
    let source = "scene start:\n  mary \"Good evening, {name}.\"\n";
    let mut catalog = Catalog::new("pt-BR");
    catalog.refresh(&strings(source), &[]);
    let file = catalog.story.get_mut("01_box.story").unwrap();
    for entry in file.values_mut() {
        entry.text = "Boa noite, {name}.".into();
    }

    let mut vm = StoryVm::from_program(program(source));
    vm.set_catalog(Some(catalog));
    vm.set_variable("name", vn_script::Value::String("Hugo".into()))
        .unwrap();

    assert_eq!(
        vm.advance(),
        Event::Say {
            speaker: Some("mary".into()),
            text: "Boa noite, Hugo.".into(),
        }
    );
}

#[test]
fn a_vm_with_no_catalog_reads_the_source() {
    let mut vm = StoryVm::from_program(program(STORY));
    assert_eq!(vm.language(), None);
    assert_eq!(
        vm.advance(),
        Event::Say {
            speaker: None,
            text: "The lamps are never put out down here.".into(),
        }
    );
}
