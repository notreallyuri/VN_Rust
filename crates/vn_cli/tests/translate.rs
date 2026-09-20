use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use vn_script::translate::Catalog;
use vn_script::{CharacterDef, Schema, SchemaFile, VariableDef};

const STORY: &str =
    "scene start:\n  \"The lamps are never put out.\"\n  mary \"Your hot water, miss.\"\n";

struct Project(PathBuf);

impl Project {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "vn_cli_translate_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("story")).unwrap();

        let mut schema = Schema::default();
        schema
            .variables
            .insert("read".into(), VariableDef::bool(false));
        schema.characters.insert(
            "mary".into(),
            CharacterDef {
                name: "Mary".into(),
                images: vec!["neutral".into()],
            },
        );
        let mut file = SchemaFile::new("Test", "story", schema);
        file.entry_scene = Some("start".into());
        file.write(root.join("schema.json")).unwrap();

        let project = Self(root);
        project.story("01.story", STORY);
        project
    }

    fn story(&self, name: &str, source: &str) -> &Self {
        fs::write(self.0.join("story").join(name), source).unwrap();
        self
    }

    fn catalog(&self, language: &str) -> Catalog {
        Catalog::read(self.0.join("lang").join(format!("{}.json", language))).unwrap()
    }

    fn save(&self, language: &str, catalog: &Catalog) {
        catalog
            .write(self.0.join("lang").join(format!("{}.json", language)))
            .unwrap();
    }

    fn translate(&self, language: &str) -> (bool, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_vn"))
            .arg("translate")
            .arg(language)
            .arg(&self.0)
            .output()
            .unwrap();
        let text =
            String::from_utf8(output.stdout).unwrap() + &String::from_utf8(output.stderr).unwrap();
        (output.status.success(), text)
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn translate_writes_a_catalog_next_to_the_schema() {
    let project = Project::new();
    let (ok, output) = project.translate("pt-BR");

    assert!(ok, "{}", output);
    assert!(output.contains("lang/pt-BR.json"), "{}", output);
    assert!(output.contains("3 strings (3 new)"), "{}", output);
    assert!(output.contains("3 missing"), "{}", output);

    let catalog = project.catalog("pt-BR");
    assert_eq!(catalog.language, "pt-BR");
    assert_eq!(catalog.story["01.story"].len(), 2);
    assert_eq!(catalog.names["mary"].source, "Mary");
    assert_eq!(
        catalog.text("01.story", "Your hot water, miss."),
        None,
        "a fresh catalog has nothing translated yet"
    );
}

#[test]
fn translating_again_keeps_the_work_and_marks_what_changed() {
    let project = Project::new();
    project.translate("pt-BR");

    let mut catalog = project.catalog("pt-BR");
    for entry in catalog.story.get_mut("01.story").unwrap().values_mut() {
        entry.text = format!("[{}]", entry.source);
    }
    project.save("pt-BR", &catalog);

    project.story(
        "01.story",
        "scene start:\n  \"The lamps are never put out.\"\n  mary \"Your hot water, madam.\"\n",
    );
    let (ok, output) = project.translate("pt-BR");
    assert!(ok, "{}", output);
    assert!(output.contains("1 new"), "{}", output);
    assert!(output.contains("1 stale"), "{}", output);

    let catalog = project.catalog("pt-BR");
    assert_eq!(
        catalog.text("01.story", "The lamps are never put out."),
        Some("[The lamps are never put out.]"),
        "the line that did not change keeps its translation"
    );
    assert_eq!(catalog.text("01.story", "Your hot water, madam."), None);
    assert_eq!(catalog.stale().len(), 1);
}

#[test]
fn a_second_run_that_changes_nothing_leaves_the_file_alone() {
    let project = Project::new();
    project.translate("pt-BR");
    let (ok, output) = project.translate("pt-BR");

    assert!(ok, "{}", output);
    assert!(output.contains("unchanged:"), "{}", output);
    assert!(output.contains("(0 new)"), "{}", output);
}

#[test]
fn a_story_with_errors_extracts_nothing() {
    let project = Project::new();
    project.story(
        "01.story",
        "scene start:\n  show ghost neutral\n  remoe mary\n",
    );

    let (ok, output) = project.translate("pt-BR");
    assert!(!ok, "{}", output);
    assert!(output.contains("nothing extracted"), "{}", output);
    assert!(!project.0.join("lang").exists());
}

#[test]
fn a_language_has_to_look_like_a_language() {
    let project = Project::new();
    let (ok, output) = project.translate("../etc");

    assert!(!ok, "{}", output);
    assert!(output.contains("is not a language tag"), "{}", output);
}

#[test]
fn a_project_without_a_schema_says_so() {
    let root = std::env::temp_dir().join(format!("vn_cli_translate_bare_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vn"))
        .arg("translate")
        .arg("pt-BR")
        .arg(&root)
        .output()
        .unwrap();
    let text = String::from_utf8(output.stderr).unwrap();

    assert!(!output.status.success());
    assert!(text.contains("no schema.json"), "{}", text);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn the_catalog_lands_where_the_engine_will_look_for_it() {
    let project = Project::new();
    project.translate("ja");
    assert!(Path::new(&project.0).join("lang").join("ja.json").is_file());
}
