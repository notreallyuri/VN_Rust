use std::fs;
use std::path::PathBuf;

use novn_build::{Embed, generate};

struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assets(name: &str) -> TempDir {
    let dir = std::env::temp_dir().join(format!("vn_build_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    for (path, contents) in [
        ("story/01.story", "scene start:\n"),
        ("backgrounds/hall.png", "png"),
        ("sources/hall.psd", "psd"),
        ("schema.json", "{}"),
        (".DS_Store", "junk"),
        ("fonts/.hidden/x.ttf", "junk"),
    ] {
        let file = dir.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, contents).unwrap();
    }
    TempDir(dir)
}

#[test]
fn files_are_sorted_relative_and_filtered() {
    let dir = assets("files");
    let all: Vec<String> = Embed::new(&dir.0)
        .files(&dir.0)
        .unwrap()
        .into_iter()
        .map(|(relative, _)| relative)
        .collect();
    assert_eq!(
        all,
        [
            "backgrounds/hall.png",
            "schema.json",
            "sources/hall.psd",
            "story/01.story"
        ]
    );

    let kept: Vec<String> = Embed::new(&dir.0)
        .exclude(["schema.json", "sources"])
        .files(&dir.0)
        .unwrap()
        .into_iter()
        .map(|(relative, _)| relative)
        .collect();
    assert_eq!(kept, ["backgrounds/hall.png", "story/01.story"]);
}

#[test]
fn generated_code_is_a_static_table() {
    let dir = assets("code");
    let files = Embed::new(&dir.0)
        .exclude(["sources"])
        .files(&dir.0)
        .unwrap();
    let code = generate(&files);
    assert!(code.contains("static VN_ASSETS: &[(&str, &[u8])]"));
    assert!(code.contains("(\"story/01.story\", include_bytes!("));
    assert_eq!(code.matches("include_bytes!").count(), 3);
    assert_eq!(generate(&[]).matches("include_bytes!").count(), 0);
}
