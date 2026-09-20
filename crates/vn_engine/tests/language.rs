use std::fs;
use std::path::PathBuf;

use vn_engine::app::VnApp;
use vn_engine::data::assets::{Assets, EmbeddedFile};
use vn_engine::data::settings::Settings;
use vn_engine::game::language::{Language, load_catalog};
use vn_engine::screens::settings::SettingsConfig;
use vn_engine::screens::settings::SettingsRow;
use vn_engine::script::translate;
use vn_engine::script::translate::Catalog;

static EMBEDDED: &[EmbeddedFile] = &[(
    "lang/ja.json",
    br#"{"format_version":1,"language":"ja","story":{},"names":{}}"#,
)];

struct Dir(PathBuf);

impl Dir {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("vn_engine_lang_{}_{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("lang")).unwrap();
        Self(dir)
    }

    fn catalog(&self, code: &str, catalog: &Catalog) -> &Self {
        catalog
            .write(self.0.join("lang").join(format!("{}.json", code)))
            .unwrap();
        self
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn config(languages: &[Language]) -> SettingsConfig {
    SettingsConfig {
        languages: languages.to_vec(),
        ..SettingsConfig::default()
    }
}

fn two() -> Vec<Language> {
    vec![
        Language::source("English"),
        Language::new("pt-BR", "Português (BR)"),
    ]
}

#[test]
fn a_game_lists_the_languages_it_ships() {
    let app = VnApp::new("Test")
        .source_language("English")
        .language("pt-BR", "Português (BR)")
        .language("ja", "日本語");

    assert_eq!(
        app.languages(),
        [
            Language::source("English"),
            Language::new("pt-BR", "Português (BR)"),
            Language::new("ja", "日本語"),
        ]
    );
    assert_eq!(VnApp::new("Test").languages().len(), 1);

    let relabelled = VnApp::new("Test")
        .language("ja", "Japanese")
        .language("ja", "日本語");
    assert_eq!(
        relabelled.languages(),
        [Language::source("English"), Language::new("ja", "日本語")]
    );
}

#[test]
fn a_catalog_is_read_from_a_folder_or_from_the_executable() {
    let mut catalog = Catalog::new("pt-BR");
    catalog.refresh(
        &[translate::Source {
            file: "01.story".into(),
            line: 2,
            kind: translate::StringKind::Narration,
            speaker: None,
            text: "The lamps are never put out.".into(),
        }],
        &[],
    );
    catalog
        .story
        .get_mut("01.story")
        .unwrap()
        .values_mut()
        .next()
        .unwrap()
        .text = "As lâmpadas nunca se apagam.".into();

    let dir = Dir::new("read");
    dir.catalog("pt-BR", &catalog);

    let loaded = load_catalog(&Assets::Dir(dir.0.clone()), "pt-BR").unwrap();
    assert_eq!(loaded.language, "pt-BR");
    assert_eq!(
        loaded.text("01.story", "The lamps are never put out."),
        Some("As lâmpadas nunca se apagam.")
    );

    assert_eq!(
        load_catalog(&Assets::Embedded(EMBEDDED), "ja")
            .unwrap()
            .language,
        "ja"
    );
}

#[test]
fn a_missing_or_broken_catalog_is_an_error_not_a_panic() {
    let dir = Dir::new("broken");
    fs::write(dir.0.join("lang").join("pt-BR.json"), "{ not json").unwrap();

    let assets = Assets::Dir(dir.0.clone());
    assert!(load_catalog(&assets, "pt-BR").is_err());
    let missing = load_catalog(&assets, "fr").unwrap_err();
    assert!(missing.contains("fr.json"), "{}", missing);
}

#[test]
fn the_language_row_only_shows_when_there_is_a_choice() {
    assert!(
        !SettingsConfig::default()
            .rows()
            .contains(&SettingsRow::Language)
    );
    assert!(
        !config(&[Language::source("English")])
            .rows()
            .contains(&SettingsRow::Language)
    );
    assert!(config(&two()).rows().contains(&SettingsRow::Language));
    assert!(!SettingsRow::Language.is_slider());
}

#[test]
fn the_language_row_cycles_through_the_languages() {
    let config = config(&two());
    let mut settings = Settings::default();
    assert_eq!(settings.language, None);
    assert_eq!(
        config.value_name(SettingsRow::Language, &settings),
        "English"
    );

    config.step(SettingsRow::Language, &mut settings, 1);
    assert_eq!(settings.language.as_deref(), Some("pt-BR"));
    assert_eq!(
        config.value_name(SettingsRow::Language, &settings),
        "Português (BR)"
    );

    config.step(SettingsRow::Language, &mut settings, 1);
    assert_eq!(settings.language, None, "it wraps back to the source");

    config.step(SettingsRow::Language, &mut settings, -1);
    assert_eq!(
        settings.language.as_deref(),
        Some("pt-BR"),
        "a non-slider row steps forwards whichever way it is nudged"
    );
}

#[test]
fn a_language_the_game_no_longer_ships_still_has_a_name() {
    let config = config(&two());
    let settings = Settings {
        language: Some("de".into()),
        ..Settings::default()
    };
    assert_eq!(config.value_name(SettingsRow::Language, &settings), "de");
    assert!(config.language_of(Some("de")).is_none());
    assert_eq!(
        config.language_of(Some("pt-BR")).map(|l| l.label.as_str()),
        Some("Português (BR)")
    );
}

#[test]
fn the_language_is_remembered_in_the_settings_file() {
    let dir = Dir::new("settings");
    let path = dir.0.join("settings.json");

    let settings = Settings {
        language: Some("pt-BR".into()),
        ..Settings::default()
    };
    settings.save(&path).unwrap();
    assert_eq!(Settings::load(&path), settings);

    let json = fs::read_to_string(&path).unwrap();
    assert!(json.contains("\"language\": \"pt-BR\""), "{}", json);

    Settings::default().save(&path).unwrap();
    let json = fs::read_to_string(&path).unwrap();
    assert!(
        !json.contains("language"),
        "the source language writes no key"
    );
    assert_eq!(Settings::load(&path).language, None);
}

#[test]
fn a_hot_reload_keeps_the_language() {
    use vn_engine::data::rollback::{Rollback, RollbackConfig};
    use vn_engine::game::hot_reload::swap_story;
    use vn_engine::script::{StoryVm, compile_sources};

    let source = "scene start:\n  \"The lamps are never put out.\"\n  \"Paper is cheaper.\"\n";
    let story = |source: &str| StoryVm::from_program(compile_sources([("story/01.story", source)]));

    let mut current = story(source);
    current.set_catalog(Some(Catalog::new("pt-BR")));
    current.advance();

    let mut rollback = Rollback::new(RollbackConfig::default());
    let fresh = story(&source.replace("Paper is cheaper.", "Paper is cheaper than daylight."));
    swap_story(&mut current, fresh, &mut rollback).unwrap();

    assert_eq!(
        current.language(),
        Some("pt-BR"),
        "reloading a story mid-game must not drop the player's language"
    );
}

#[test]
fn a_message_is_filled_in_after_it_is_translated() {
    use vn_engine::ui::labels::fill;

    assert_eq!(
        fill("Saved to {slot}", &[("slot", "Slot 3")]),
        "Saved to Slot 3"
    );
    assert_eq!(
        fill("Gravado em {slot}", &[("slot", "Espaço 3")]),
        "Gravado em Espaço 3"
    );
    assert_eq!(fill("No fields here", &[]), "No fields here");
    assert_eq!(
        fill("{a} and {b}", &[("a", "one"), ("b", "two")]),
        "one and two"
    );
    assert_eq!(
        fill("{unknown} stays", &[("slot", "x")]),
        "{unknown} stays",
        "a field the caller did not supply is left as written"
    );
    assert_eq!(fill("half {open", &[("open", "x")]), "half {open");
}

#[test]
fn the_screens_offer_their_own_strings_for_translation() {
    let app = VnApp::new("Test")
        .language("pt-BR", "Português (BR)")
        .ui_text("Filed in the archive");
    let strings = app.ui_strings().strings;
    let has = |text: &str| strings.iter().any(|s| s == text);

    for text in [
        "Settings",
        "Back",
        "New Game",
        "Continue",
        "Quit",
        "Resume",
        "Save",
        "Load",
        "Text speed",
        "Language",
        "Quick saved",
        "Saved to {slot}",
        "Slot {number}",
        "Overwrite {slot}?",
        "The End",
        "Log",
        "Auto",
    ] {
        assert!(has(text), "{} is not offered for translation", text);
    }

    assert!(has("Português (BR)"), "language names are translatable too");
    assert!(
        has("Filed in the archive"),
        "a game can add its own strings"
    );
    assert!(!has(""), "no blank entries");

    let mut sorted = strings.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(strings, sorted, "the list is sorted and free of repeats");
}

#[test]
fn a_game_that_renames_a_label_offers_the_new_name() {
    let app = VnApp::new("Test").main_menu(|m| {
        m.button("Begin the Archive", vn_engine::action::Action::NewGame)
            .button("Leave", vn_engine::action::Action::Quit)
    });
    let strings = app.ui_strings().strings;

    assert!(strings.iter().any(|s| s == "Begin the Archive"));
    assert!(
        !strings.iter().any(|s| s == "New Game"),
        "a replaced label is not offered; what the game actually shows is"
    );
}

fn translated(file: &str, source: &str, into: &str) -> Catalog {
    let mut catalog = Catalog::new("pt-BR");
    catalog.refresh(
        &[translate::Source {
            file: file.into(),
            line: 1,
            kind: translate::StringKind::Dialogue,
            speaker: None,
            text: source.into(),
        }],
        &[],
    );
    for entry in catalog.story.get_mut(file).unwrap().values_mut() {
        entry.text = into.to_string();
    }
    catalog
}

#[test]
fn a_logged_line_is_read_in_whatever_language_is_active() {
    use vn_engine::data::session::Spoken;

    let said = Spoken::with_variables("01.story", "Sit down.", &Default::default());
    assert_eq!(said.text(None), "Sit down.");

    let catalog = translated("01.story", "Sit down.", "Sente-se.");
    assert_eq!(
        said.text(Some(&catalog)),
        "Sente-se.",
        "a line already in the log must follow a language change"
    );

    let other = translated("01.story", "Something else.", "Outra coisa.");
    assert_eq!(
        said.text(Some(&other)),
        "Sit down.",
        "an untranslated line falls back to the source text"
    );
}

#[test]
fn a_translated_line_keeps_the_values_it_was_read_with() {
    use vn_engine::data::session::Spoken;
    use vn_engine::script::Value;

    let source = "Welcome, {player_name}.";
    let mut variables = std::collections::HashMap::new();
    variables.insert("player_name".to_string(), Value::String("Mary".into()));
    let said = Spoken::with_variables("01.story", source, &variables);

    let catalog = translated("01.story", source, "Bem-vinda, {player_name}.");
    assert_eq!(said.text(Some(&catalog)), "Bem-vinda, Mary.");
}
