use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use vn_script::SchemaFile;

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "vn_cli_new_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.0.join(relative)).unwrap()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vn(args: &[&str]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_vn"))
        .args(args)
        .output()
        .unwrap();
    let text =
        String::from_utf8(output.stdout).unwrap() + &String::from_utf8(output.stderr).unwrap();
    (output.status.success(), text)
}

#[test]
fn creates_a_project_that_passes_vn_check() {
    let temp = TempDir::new();
    let dir = temp.0.join("my-first-novel");
    let (ok, output) = vn(&["new", dir.to_str().unwrap()]);
    assert!(ok, "{}", output);
    assert!(output.contains("Created My First Novel (my_first_novel)"));

    for file in [
        "Cargo.toml",
        "build.rs",
        ".gitignore",
        "README.md",
        "src/main.rs",
        "assets/story/start.story",
        "assets/schema.json",
        "assets/characters/.gitkeep",
        "assets/backgrounds/.gitkeep",
        "assets/fonts/.gitkeep",
    ] {
        assert!(dir.join(file).is_file(), "missing {}", file);
    }

    let (ok, output) = vn(&["check", dir.join("assets").to_str().unwrap()]);
    assert!(ok, "{}", output);
    assert!(output.contains("0 errors, 0 warnings"), "{}", output);
}

#[test]
fn cargo_toml_names_the_package_and_points_at_the_engine() {
    let temp = TempDir::new();
    let dir = temp.0.join("Night Shift");
    assert!(vn(&["new", dir.to_str().unwrap()]).0);

    let cargo = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
    assert!(cargo.contains("name = \"night_shift\""));
    assert!(cargo.contains("\n[workspace]\n"));

    let engine = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../vn_engine")
        .canonicalize()
        .unwrap();
    let path = cargo
        .lines()
        .find_map(|line| line.strip_prefix("vn_engine = { path = \""))
        .and_then(|rest| rest.strip_suffix("\" }"))
        .expect("a path dependency");
    assert_eq!(dir.join(path).canonicalize().unwrap(), engine);
    assert!(cargo.contains("[build-dependencies]\nvn_build = { path = \""));
}

#[test]
fn title_and_engine_git_flags() {
    let temp = TempDir::new();
    let dir = temp.0.join("game");
    let (ok, output) = vn(&[
        "new",
        dir.to_str().unwrap(),
        "--title",
        "The \"Quoted\" Game",
        "--engine-git",
        "https://example.com/vn.git",
    ]);
    assert!(ok, "{}", output);

    let temp_project = TempDir(dir);
    assert!(
        temp_project
            .read("Cargo.toml")
            .contains("vn_engine = { git = \"https://example.com/vn.git\" }")
    );
    assert!(
        temp_project
            .read("Cargo.toml")
            .contains("vn_build = { git = \"https://example.com/vn.git\" }")
    );
    assert!(
        temp_project
            .read("src/main.rs")
            .contains("VnApp::new(\"The \\\"Quoted\\\" Game\")")
    );
    assert!(
        temp_project
            .read("README.md")
            .starts_with("# The \"Quoted\" Game\n")
    );
    let schema = SchemaFile::from_json(&temp_project.read("assets/schema.json")).unwrap();
    assert_eq!(schema.game, "The \"Quoted\" Game");
}

#[test]
fn refuses_a_non_empty_directory() {
    let temp = TempDir::new();
    fs::write(temp.0.join("notes.txt"), "keep me").unwrap();

    let (ok, output) = vn(&["new", temp.0.to_str().unwrap()]);
    assert!(!ok);
    assert!(
        output.contains("already exists and is not empty"),
        "{}",
        output
    );
    assert_eq!(temp.read("notes.txt"), "keep me");
    assert!(!temp.0.join("Cargo.toml").exists());
}

#[test]
fn an_empty_directory_is_fine() {
    let temp = TempDir::new();
    let dir = temp.0.join("empty");
    fs::create_dir_all(&dir).unwrap();
    assert!(vn(&["new", dir.to_str().unwrap()]).0);
    assert!(dir.join("src/main.rs").is_file());
}

#[test]
fn rejects_names_that_are_not_crate_names() {
    let temp = TempDir::new();
    let (ok, output) = vn(&["new", temp.0.join("123").to_str().unwrap()]);
    assert!(!ok);
    assert!(
        output.contains("can't be turned into a crate name"),
        "{}",
        output
    );
}

#[test]
fn bad_arguments_print_usage() {
    for args in [
        &["new"][..],
        &["new", "a", "b"],
        &["new", "a", "--title"],
        &["new", "a", "--frobnicate"],
    ] {
        let (ok, output) = vn(args);
        assert!(!ok, "{:?}", args);
        assert!(output.contains("usage:"), "{:?}: {}", args, output);
    }
}
