use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Stories(PathBuf);

impl Stories {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "vn_cli_fmt_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn story(&self, name: &str, source: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, source).unwrap();
        path
    }
}

impl Drop for Stories {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fmt(args: &[&Path]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_vn"))
        .arg("fmt")
        .args(args)
        .output()
        .unwrap();
    let text =
        String::from_utf8(output.stdout).unwrap() + &String::from_utf8(output.stderr).unwrap();
    (output.status.success(), text)
}

#[test]
fn formatting_rewrites_the_file_and_is_stable() {
    let stories = Stories::new();
    let path = stories.story(
        "messy.story",
        "scene start:\n        \"a\"\n        set x=1\n",
    );

    let (ok, out) = fmt(&[&path]);
    assert!(ok, "{}", out);
    assert!(out.contains("formatted"), "{}", out);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "scene start:\n  \"a\"\n  set x = 1\n"
    );

    let (ok, out) = fmt(&[&path]);
    assert!(ok, "{}", out);
    assert!(out.contains("0 reformatted"), "{}", out);
}

#[test]
fn check_reports_without_writing_and_fails() {
    let stories = Stories::new();
    let source = "scene start:\n   \"a\"\n";
    let path = stories.story("messy.story", source);

    let (ok, out) = fmt(&[&path, Path::new("--check")]);
    assert!(!ok, "{}", out);
    assert!(out.contains("would reformat"), "{}", out);
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
}

#[test]
fn check_passes_on_a_formatted_file() {
    let stories = Stories::new();
    let path = stories.story("tidy.story", "scene start:\n  \"a\"\n");

    let (ok, out) = fmt(&[&path, Path::new("--check")]);
    assert!(ok, "{}", out);
    assert!(out.contains("0 to reformat"), "{}", out);
}

#[test]
fn a_file_with_errors_is_left_alone() {
    let stories = Stories::new();
    let source = "scene start:\n  show\n";
    let path = stories.story("broken.story", source);

    let (ok, out) = fmt(&[&path]);
    assert!(!ok, "{}", out);
    assert!(out.contains("it has errors"), "{}", out);
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
}

#[test]
fn a_directory_formats_every_story_in_it() {
    let stories = Stories::new();
    stories.story("a.story", "scene a:\n   \"a\"\n");
    stories.story("b.story", "scene b:\n  \"b\"\n");

    let (ok, out) = fmt(&[&stories.0]);
    assert!(ok, "{}", out);
    assert!(out.contains("2 files, 1 reformatted"), "{}", out);
    assert_eq!(
        fs::read_to_string(stories.0.join("a.story")).unwrap(),
        "scene a:\n  \"a\"\n"
    );
}
