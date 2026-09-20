use std::fs;
use std::path::PathBuf;

use vn_engine::screens::SettingsConfig;
use vn_engine::script::translate::{self, Catalog};
use vn_engine::{Assets, EmbeddedFile, Language, Settings, SettingsRow, VnApp, load_catalog};

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
    use vn_engine::script::{StoryVm, compile_sources};
    use vn_engine::{Rollback, RollbackConfig, swap_story};

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
