#![cfg(feature = "character-visuals")]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use novn::app::AppError;
use novn::data::assets::{Assets, EmbeddedFile};
use novn::game::hot_reload::StoryLoader;
use novn::game::puppet::{BLINK, BREATH, Motion, Part, Pose, Puppet, SWAY};
use novn::game::visuals::VisualRegistry;
use novn::script::{CharacterDef, Schema};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/puppet");
const PARTS: [&str; 5] = [
    "body.png",
    "head.png",
    "eyes.png",
    "mouth.png",
    "mouth_smile.png",
];

struct Dir(PathBuf);

impl Dir {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "vn_engine_visuals_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("story")).unwrap();
        fs::create_dir_all(dir.join("characters/mary")).unwrap();
        for part in PARTS {
            let from = PathBuf::from(FIXTURES).join("characters/mary").join(part);
            fs::copy(from, dir.join("characters/mary").join(part)).unwrap();
        }
        Self(dir)
    }

    fn story(&self, source: &str) -> &Self {
        fs::write(self.0.join("story/01.story"), source).unwrap();
        self
    }

    fn drop_part(&self, part: &str) {
        fs::remove_file(self.0.join("characters/mary").join(part)).unwrap();
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn rig() -> Puppet {
    Puppet::new(600.0, 900.0)
        .folder("mary")
        .part(Part::new("body").at(300.0, 900.0).pivot(0.5, 1.0))
        .part(
            Part::new("head")
                .at(300.0, 400.0)
                .pivot(0.5, 1.0)
                .bind(SWAY, Motion::rotate(5.0))
                .bind(BREATH, Motion::offset(0.0, -6.0)),
        )
        .part(
            Part::new("eyes")
                .at(300.0, 250.0)
                .bind(BLINK, Motion::scale(1.0, 0.08)),
        )
        .part(
            Part::new("mouth")
                .at(300.0, 330.0)
                .bind("mouth", Motion::scale(1.0, 2.4)),
        )
        .appearance("neutral", Pose::new())
        .appearance("happy", Pose::new().swap("mouth", "mouth_smile"))
}

fn schema(images: &[&str]) -> Schema {
    let mut schema = Schema::default();
    schema.characters.insert(
        "mary".into(),
        CharacterDef {
            name: "Mary".into(),
            images: images.iter().map(|image| image.to_string()).collect(),
        },
    );
    schema
}

fn loader(dir: &Dir, schema: Schema, visuals: VisualRegistry) -> StoryLoader {
    StoryLoader {
        visuals,
        assets: dir.0.clone().into(),
        story_dir: "story".into(),
        schema,
        entry_scene: Some("start".into()),
        warn_missing_art: true,
    }
}

fn registry(character: &str, puppet: Puppet) -> VisualRegistry {
    let mut visuals = VisualRegistry::default();
    visuals.insert(character.into(), puppet);
    visuals
}

#[test]
fn a_rigs_appearances_are_what_the_story_may_show() {
    let dir = Dir::new();
    dir.story("scene start:\n  show mary happy\n  \"x\"\n");
    let visuals = registry("mary", rig());

    let Err(AppError::Script { errors, .. }) =
        loader(&dir, schema(&["sad", "sleepy"]), visuals.clone()).load()
    else {
        panic!("a character that declares its images cannot be shown as another one");
    };
    assert!(errors[0].message.contains("happy"), "{}", errors[0].message);

    let mut extended = schema(&["sad", "happy"]);
    visuals.extend_schema(&mut extended);
    assert_eq!(
        extended.characters["mary"].images,
        ["happy", "neutral", "sad"],
        "the rig's appearances join the character's own images, sorted and deduplicated"
    );

    let (story, warnings) = loader(&dir, extended, visuals).load().unwrap();
    assert_eq!(story.entry_scene(), Some("start"));
    assert!(
        warnings.is_empty(),
        "an appearance the backend draws needs no PNG: {warnings:?}"
    );
}

#[test]
fn art_warnings_skip_what_a_backend_draws_and_still_name_missing_pngs() {
    let dir = Dir::new();
    dir.story("scene start:\n  show mary happy\n  \"x\"\n  show mary sad\n  \"y\"\n");
    let visuals = registry("mary", rig());
    let mut schema = schema(&["sad"]);
    visuals.extend_schema(&mut schema);

    let (_, warnings) = loader(&dir, schema, visuals).load().unwrap();
    let messages: Vec<_> = warnings.iter().map(|w| w.message.clone()).collect();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(
        messages[0].contains("characters/mary/sad.png"),
        "{messages:?}"
    );
}

#[test]
fn a_rig_is_checked_against_the_assets_before_the_window_opens() {
    let dir = Dir::new();
    dir.story("scene start:\n  \"x\"\n");
    let visuals = registry("mary", rig());
    let mut schema = schema(&[]);
    visuals.extend_schema(&mut schema);

    assert!(loader(&dir, schema.clone(), visuals.clone()).load().is_ok());

    dir.drop_part("mouth_smile.png");
    let Err(AppError::Visual(error)) = loader(&dir, schema, visuals).load() else {
        panic!("a missing part is a startup error");
    };
    assert!(error.starts_with("character 'mary':"), "{error}");
    assert!(error.contains("mouth_smile.png"), "{error}");
    assert!(
        format!("{}", AppError::Visual(error)).starts_with("invalid character visual:"),
        "the error says what kind of thing failed"
    );
}

#[test]
fn a_backend_for_a_character_nobody_declared_is_refused() {
    let dir = Dir::new();
    dir.story("scene start:\n  \"x\"\n");
    let visuals = registry("ghost", rig());
    let mut schema = schema(&[]);
    visuals.extend_schema(&mut schema);

    let Err(AppError::Visual(error)) = loader(&dir, schema, visuals).load() else {
        panic!("expected a visual error");
    };
    assert!(error.contains("unregistered character 'ghost'"), "{error}");
}

#[test]
fn appearance_names_have_to_be_story_identifiers() {
    let dir = Dir::new();
    dir.story("scene start:\n  \"x\"\n");
    let named = |name: &str| {
        let puppet = Puppet::new(600.0, 900.0)
            .folder("mary")
            .part(Part::new("body"))
            .appearance(name, Pose::new());
        let visuals = registry("mary", puppet);
        match loader(&dir, schema(&[]), visuals).load() {
            Err(AppError::Visual(error)) => error,
            other => panic!("expected a visual error, got {:?}", other.map(|_| ())),
        }
    };

    for name in ["not an id", "1st", ""] {
        let error = named(name);
        assert!(
            error.contains("appearance names must be unique story identifiers"),
            "{name:?}: {error}"
        );
    }
}

#[test]
fn a_rig_loads_the_same_pictures_from_a_folder_and_from_an_embedded_build() {
    static FILES: &[EmbeddedFile] = &[
        (
            "characters/mary/body.png",
            include_bytes!("fixtures/puppet/characters/mary/body.png"),
        ),
        (
            "characters/mary/head.png",
            include_bytes!("fixtures/puppet/characters/mary/head.png"),
        ),
        (
            "characters/mary/eyes.png",
            include_bytes!("fixtures/puppet/characters/mary/eyes.png"),
        ),
        (
            "characters/mary/mouth.png",
            include_bytes!("fixtures/puppet/characters/mary/mouth.png"),
        ),
        (
            "characters/mary/mouth_smile.png",
            include_bytes!("fixtures/puppet/characters/mary/mouth_smile.png"),
        ),
    ];

    let folder = Assets::from(PathBuf::from(FIXTURES));
    let embedded = Assets::Embedded(FILES);
    assert!(rig().check(&folder).is_ok());
    assert!(rig().check(&embedded).is_ok());

    let short = Assets::Embedded(&FILES[..4]);
    let error = rig().check(&short).unwrap_err();
    assert!(error.contains("mouth_smile.png"), "{error}");
    assert!(error.contains("is missing"), "{error}");
}
