#[cfg(feature = "engine")]
const STORY: &str = r#"scene start:
  show kaede neutral at center with dissolve
  kaede "A model on the story path: show kaede neutral."
  show kaede cheerful with dissolve
  kaede "A second appearance is a second model; the first fades out and is dropped."
  show hiyori neutral at left with dissolve
  hiyori "Two models draw at once."
  call push
  kaede "Her eyes are held shut and her mouth driven, from a frame hook, over a running motion."
  call snapshot
  kaede "That frame was taken with the push still on."
  call release
  kaede "Released, the motion has them back."
  call snapshot
  kaede "And that one after it."
  call keep
  kaede "Saved. Loading restarts the models on screen instead of dropping them."
  call reload
  kaede "Back: the same moc and textures, the appearance's motion started again."
  call snapshot
  remove hiyori with dissolve
  kaede "One model left."
  call finish
"#;

#[cfg(feature = "engine")]
fn main() -> Result<(), vn_engine::app::AppError> {
    use std::cell::Cell;
    use std::fs;
    use std::path::PathBuf;
    use std::rc::Rc;

    use vn_engine::prelude::*;
    use vn_live2d::{Appearance, Live2dCharacter};

    const SLOT: &str = "probe";
    const MOUTH: &str = "ParamMouthOpenY";
    const EYES: [&str; 2] = ["ParamEyeLOpen", "ParamEyeROpen"];

    let args: Vec<String> = std::env::args().skip(1).collect();
    let smoke = args.iter().any(|arg| arg == "--smoke");
    let broken = args.iter().any(|arg| arg == "--break");
    let model_dir = args
        .iter()
        .find(|arg| !arg.starts_with("--"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../examples/example_vn/assets/characters/kaede"
            ))
        });
    let model = fs::read_dir(&model_dir)
        .unwrap_or_else(|e| panic!("{}: {e}", model_dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .find(|name| name.ends_with(".model3.json"))
        .unwrap_or_else(|| panic!("no .model3.json in {}", model_dir.display()));

    let root = std::env::temp_dir().join(format!("vn-live2d-story-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("story")).unwrap();
    fs::create_dir_all(root.join("characters")).unwrap();
    for id in ["kaede", "hiyori"] {
        std::os::unix::fs::symlink(&model_dir, root.join("characters").join(id)).unwrap();
    }
    fs::write(root.join("story/demo.story"), STORY).unwrap();
    if broken {
        static FALLBACK: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../vn_engine/tests/fixtures/puppet/characters/mary/body.png"
        ));
        let kaede = root.join("characters/kaede");
        fs::remove_file(&kaede).unwrap();
        fs::create_dir_all(&kaede).unwrap();
        for entry in fs::read_dir(&model_dir).unwrap().filter_map(Result::ok) {
            std::os::unix::fs::symlink(entry.path(), kaede.join(entry.file_name())).unwrap();
        }
        let moc = kaede.join(model.replace(".model3.json", ".moc3"));
        let mut bytes = fs::read(&moc).unwrap();
        bytes.truncate(bytes.len() / 2);
        fs::remove_file(&moc).unwrap();
        fs::write(&moc, bytes).unwrap();
        for appearance in ["neutral", "cheerful"] {
            fs::write(kaede.join(format!("{appearance}.png")), FALLBACK).unwrap();
        }
        println!(
            "Broken run: kaede's moc3 is truncated, so every instance of her should fall back"
        );
    }
    println!("Model: {}/{model}", model_dir.display());

    let character = |id: &str| {
        Live2dCharacter::new(format!("characters/{id}/{model}"))
            .appearance("neutral", Appearance::new().motion("Idle", 0, true))
            .appearance("cheerful", Appearance::new().motion("Idle", 4, true))
    };

    let reloaded = Cell::new(false);
    let pushing = Rc::new(Cell::new(false));
    let start = Rc::clone(&pushing);
    let stop = Rc::clone(&pushing);
    let clock = Cell::new(0.0f32);

    VnApp::new("Live2D story")
        .assets(root.clone())
        .schema_file(None)
        .saves_dir(root.join("saves"))
        .initial_screen(ScreenState::Playing)
        .audio(|audio| audio.enabled(false))
        .character("kaede", Character::new("Kaede"))
        .character("hiyori", Character::new("Hiyori"))
        .character_visual("kaede", character("kaede"))
        .character_visual("hiyori", character("hiyori"))
        .on_scene_enter(|ctx, _| {
            ctx.modes.auto = true;
            None
        })
        .command_as("push", move |_, (): ()| {
            start.set(true);
            println!("Pushing her eyes shut and her mouth open from the frame hook");
            None
        })
        .command_as("release", move |ctx, (): ()| {
            stop.set(false);
            for id in EYES {
                ctx.visual_parameter("kaede", id, None);
            }
            ctx.visual_parameter("kaede", MOUTH, None);
            println!("Released them");
            None
        })
        .command_as("snapshot", |ctx, (): ()| {
            ctx.screenshot();
            None
        })
        .command_as("keep", move |ctx, (): ()| {
            match ctx.saves.save(SLOT, ctx.story, ctx.state) {
                Ok(()) => println!("Saved"),
                Err(e) => println!("Save failed: {e}"),
            }
            None
        })
        .command_as("reload", move |ctx, (): ()| {
            if reloaded.replace(true) {
                return None;
            }
            match ctx.load(SLOT) {
                Ok(_) => println!("Loaded: the models on screen were restarted"),
                Err(e) => println!("Load failed: {e}"),
            }
            None
        })
        .command_as("finish", move |_, (): ()| {
            println!("The scene finished with the models still up");
            smoke.then_some(ScreenState::Quit)
        })
        .on_frame(move |ctx, seconds| {
            if !pushing.get() {
                return;
            }
            clock.set(clock.get() + seconds);
            let open = ((clock.get() * 9.0).sin() + 1.0) / 2.0;
            for id in EYES {
                ctx.visual_parameter("kaede", id, Some(0.0));
            }
            ctx.visual_parameter("kaede", MOUTH, Some(open));
        })
        .run()
}

#[cfg(not(feature = "engine"))]
fn main() {
    eprintln!("Enable the engine feature to run this example");
}
