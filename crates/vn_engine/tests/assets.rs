use std::fs;
use std::path::PathBuf;

use vn_engine::app::VnApp;
use vn_engine::data::assets::{Assets, EmbeddedFile};

static FILES: &[EmbeddedFile] = &[
    ("backgrounds/hall.png", b"png"),
    ("story/01_start.story", b"scene start:\n  \"Hello.\"\n"),
    ("story/sub/02_more.story", b"scene more:\n  \"More.\"\n"),
    ("story/notes.txt", b"not a story"),
    ("storyteller.txt", b"not in story/"),
];

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("vn_assets_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, bytes) in FILES {
            let file = dir.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, bytes).unwrap();
        }
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn directories_and_embedded_files_answer_the_same_questions() {
    let dir = TempDir::new("same");
    for assets in [Assets::Dir(dir.0.clone()), Assets::Embedded(FILES)] {
        assert_eq!(&*assets.read("backgrounds/hall.png").unwrap(), b"png");
        assert_eq!(&*assets.read("./backgrounds\\hall.png").unwrap(), b"png");
        assert!(assets.read("backgrounds/missing.png").is_err());
        assert!(assets.exists("story/01_start.story"));
        assert!(!assets.exists("story"));
        assert_eq!(
            assets.files_under("story").unwrap(),
            [
                "story/01_start.story",
                "story/notes.txt",
                "story/sub/02_more.story"
            ]
        );
        assert_eq!(assets.files_under("").unwrap().len(), FILES.len());
    }
}

#[test]
fn embedded_paths_are_described_without_a_directory() {
    assert_eq!(
        Assets::Embedded(FILES).describe("story/01_start.story"),
        "<embedded>/story/01_start.story"
    );
    assert!(Assets::Embedded(FILES).dir().is_none());
    assert!(Assets::Embedded(FILES).is_embedded());
    assert_eq!(
        Assets::from("assets").dir(),
        Some(PathBuf::from("assets").as_path())
    );
}

#[test]
fn an_embedded_story_loads_without_any_files_on_disk() {
    let app = VnApp::new("Embedded")
        .assets("/definitely/not/a/directory")
        .embedded_assets(FILES)
        .warn_missing_art(false);
    assert!(app.asset_source().is_embedded());

    let (story, warnings) = app.check().unwrap();
    assert!(warnings.is_empty());
    let files = &story.program().files;
    assert_eq!(files.len(), 2);
    assert!(files[0].ends_with("story/01_start.story"), "{:?}", files);
}

#[test]
fn debug_builds_prefer_the_directory_when_it_exists() {
    let dir = TempDir::new("prefer");
    let app = VnApp::new("Both").assets(&dir.0).embedded_assets(FILES);
    assert_eq!(app.asset_source().is_embedded(), !cfg!(debug_assertions));

    let empty = VnApp::new("None").assets(&dir.0).embedded_assets(&[]);
    assert!(!empty.asset_source().is_embedded());
}
