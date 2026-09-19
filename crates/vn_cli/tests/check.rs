use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use vn_script::{CharacterDef, Schema, SchemaFile, VariableDef};

struct Project(PathBuf);

impl Project {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "vn_cli_check_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("story")).unwrap();
        Self(root)
    }

    fn with_schema(self) -> Self {
        let mut schema = Schema::default();
        schema
            .variables
            .insert("affection".into(), VariableDef::int(0));
        schema.characters.insert(
            "mary".into(),
            CharacterDef {
                name: "Mary".into(),
                images: vec!["neutral".into()],
            },
        );
        let mut file = SchemaFile::new("Test", "story", schema);
        file.entry_scene = Some("start".into());
        file.write(self.0.join("schema.json")).unwrap();
        self
    }

    fn story(&self, name: &str, source: &str) -> &Self {
        fs::write(self.0.join("story").join(name), source).unwrap();
        self
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vn(args: &[&Path]) -> (bool, Vec<String>) {
    let output = Command::new(env!("CARGO_BIN_EXE_vn"))
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let lines = stdout
        .lines()
        .chain(stderr.lines())
        .map(String::from)
        .collect();
    (output.status.success(), lines)
}

fn check(path: &Path) -> (bool, Vec<String>) {
    vn(&[Path::new("check"), path])
}

fn file_name(line: &str) -> &str {
    let path = line.split(':').next().unwrap();
    Path::new(path).file_name().unwrap().to_str().unwrap()
}

#[test]
fn a_valid_project_passes() {
    let project = Project::new().with_schema();
    project
        .story(
            "01.story",
            "scene start:\n  show mary neutral\n  jump two\n",
        )
        .story(
            "02.story",
            "scene two:\n  mary \"Hi.\"\n  add affection += 1\n",
        );

    for path in [
        project.path(""),
        project.path("story"),
        project.path("schema.json"),
    ] {
        let (ok, lines) = check(&path);
        assert!(ok, "{:?}", lines);
        assert_eq!(lines.len(), 1, "{:?}", lines);
        assert!(lines[0].starts_with("2 files, 2 scenes checked against "));
        assert!(lines[0].ends_with("schema.json: 0 errors, 0 warnings"));
    }
}

#[test]
fn registry_and_syntax_errors_fail_with_their_file() {
    let project = Project::new().with_schema();
    project
        .story("01.story", "scene start:\n  set affecton = 1\n  jump two\n")
        .story("02.story", "scene two:\n  marry \"Hi.\"\n  show mary\n");

    let (ok, lines) = check(&project.path(""));
    assert!(!ok);
    assert_eq!(lines.len(), 4, "{:?}", lines);
    assert_eq!(file_name(&lines[0]), "01.story");
    assert!(
        lines[0].ends_with(":2: error: unknown variable 'affecton'; did you mean 'affection'?")
    );
    assert!(
        lines[1].ends_with(
            "02.story:2: error: speaker: unknown character 'marry'; did you mean 'mary'?"
        )
    );
    assert!(
        lines[2].ends_with("02.story:3: error: `show mary` needs an image: `show mary <image>`")
    );
    assert!(lines[3].ends_with("3 errors, 0 warnings"));
}

#[test]
fn one_file_is_checked_in_its_project() {
    let project = Project::new().with_schema();
    project
        .story("01.story", "scene start:\n  set nope = 1\n  jump two\n")
        .story("02.story", "scene two:\n  mary \"Hi.\"\n");

    let (ok, lines) = check(&project.path("story/02.story"));
    assert!(ok, "{:?}", lines);
    assert_eq!(lines.len(), 1, "{:?}", lines);
    assert!(lines[0].contains("0 errors, 0 warnings in "));

    let (ok, lines) = check(&project.path("story/01.story"));
    assert!(!ok);
    assert_eq!(lines.len(), 2, "{:?}", lines);
    assert!(lines[0].ends_with("01.story:2: error: unknown variable 'nope'"));
}

#[test]
fn entry_scene_comes_from_the_schema() {
    let project = Project::new().with_schema();
    project.story("01.story", "scene other:\n  \"x\"\n");

    let (ok, lines) = check(&project.path(""));
    assert!(!ok);
    assert_eq!(lines[0], "error: entry scene 'start' does not exist");
}

#[test]
fn a_file_outside_the_story_dir_uses_the_registries_only() {
    let project = Project::new().with_schema();
    project.story("01.story", "scene start:\n  \"x\"\n");
    fs::write(project.path("loose.story"), "scene loose:\n  bob \"Hi.\"\n").unwrap();

    let (ok, lines) = check(&project.path("loose.story"));
    assert!(!ok);
    assert!(lines[0].ends_with("loose.story:2: error: speaker: unknown character 'bob'"));
    assert!(lines[1].starts_with("1 file, 1 scene checked against "));
}

#[test]
fn without_a_schema_only_syntax_is_checked() {
    let project = Project::new();
    project.story("a.story", "scene a:\n  bob \"Hi.\"\n  show bob\n");

    let (ok, lines) = check(&project.path("story"));
    assert!(!ok);
    assert_eq!(lines.len(), 2, "{:?}", lines);
    assert!(lines[0].ends_with("a.story:3: error: `show bob` needs an image: `show bob <image>`"));
    assert!(lines[1].contains("without a schema (syntax only"));
}

#[test]
fn explicit_schema_option() {
    let project = Project::new().with_schema();
    project.story("01.story", "scene start:\n  bob \"Hi.\"\n");
    let elsewhere = Project::new();
    elsewhere.story("a.story", "scene start:\n  bob \"Hi.\"\n");

    let (ok, lines) = vn(&[
        Path::new("check"),
        &elsewhere.path("story/a.story"),
        Path::new("--schema"),
        &project.path("schema.json"),
    ]);
    assert!(!ok);
    assert!(lines[0].ends_with("a.story:2: error: speaker: unknown character 'bob'"));
}

#[test]
fn unreadable_inputs() {
    let project = Project::new();

    let (ok, lines) = check(&project.path("missing"));
    assert!(!ok);
    assert!(
        lines[0].contains("no such file or directory"),
        "{:?}",
        lines
    );

    let (ok, lines) = check(&project.path("story"));
    assert!(!ok);
    assert!(lines[0].contains("no .story files in"), "{:?}", lines);

    fs::write(project.path("schema.json"), "{ \"format_version\": 99 }").unwrap();
    project.story("a.story", "scene a:\n  \"x\"\n");
    let (ok, lines) = check(&project.path("story"));
    assert!(!ok);
    assert!(lines[0].contains("schema.json"), "{:?}", lines);
}

#[test]
fn dump_accepts_a_directory() {
    let project = Project::new();
    project
        .story("01.story", "scene start:\n  jump two\n")
        .story("02.story", "scene two:\n  \"x\"\n");

    let (ok, lines) = vn(&[Path::new("dump"), &project.path("story")]);
    assert!(ok, "{:?}", lines);
    assert_eq!(lines[0], "2 files, 2 scenes, 4 instructions");
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("scene two:") && l.ends_with("02.story:1)"))
    );
}
