#[cfg(feature = "engine")]
const STORY: &str = r#"scene start:
  show one neutral at center with dissolve
  one "A model on the story path: show one neutral."
  show one cheerful with dissolve
  one "A second appearance is a second model, with its own motion and expression."
  show two neutral at left with dissolve
  two "Two models draw at once."
  call push
  one "Eyes held shut and a mouth driven from a frame hook, over a running motion."
  call snapshot
  one "That frame was taken with the push still on."
  call keep
  one "Saved, with the push in the save."
  call release
  one "Released, the motion has them back."
  call snapshot
  one "And that one after it. Loading now should bring the push back with it."
  call reload
  one "Back: the same moc and textures, the appearance's preset started again."
  call snapshot
  call post
  one "A shader pass is running over the finished frame, models included."
  call snapshot
  call plain
  remove two with dissolve
  one "One model left."
  show one neutral with dissolve
  one "Its first appearance again, with that appearance's own expression."
  show one cheerful with dissolve
  one "And its second. Page Up walks back through all of this."
  call finish
"#;

#[cfg(feature = "engine")]
fn main() -> Result<(), vn_engine::app::AppError> {
    use std::cell::{Cell, RefCell};
    use std::fs;
    use std::path::PathBuf;
    use std::rc::Rc;

    use vn_engine::prelude::*;
    use vn_live2d::{Appearance, Live2dCharacter};

    const SLOT: &str = "probe";

    let args: Vec<String> = std::env::args().skip(1).collect();
    let smoke = args.iter().any(|arg| arg == "--smoke");
    let broken = args.iter().any(|arg| arg == "--break");
    let post = args.iter().any(|arg| arg == "--post");
    let watch = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--watch="))
        .and_then(|seconds| seconds.parse::<f32>().ok());
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

    let settings: vn_engine::serde_json::Value =
        vn_engine::serde_json::from_slice(&fs::read(model_dir.join(&model)).unwrap()).unwrap();
    let references = &settings["FileReferences"];
    let idles = references["Motions"]["Idle"]
        .as_array()
        .map_or(0, |group| group.len());
    let expressions: Vec<String> = references["Expressions"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|item| item["Name"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let parameters: Vec<String> = references["DisplayInfo"]
        .as_str()
        .and_then(|file| fs::read(model_dir.join(file)).ok())
        .and_then(|bytes| {
            vn_engine::serde_json::from_slice::<vn_engine::serde_json::Value>(&bytes).ok()
        })
        .and_then(|info| {
            info["Parameters"].as_array().map(|list| {
                list.iter()
                    .filter_map(|item| item["Id"].as_str().map(str::to_string))
                    .collect()
            })
        })
        .unwrap_or_default();
    let has = |id: &str| parameters.is_empty() || parameters.iter().any(|have| have == id);
    let pushed: Vec<String> = ["ParamEyeLOpen", "ParamEyeROpen"]
        .into_iter()
        .chain(
            ["ParamMouthOpenY", "ParamMouthUp", "ParamMouthOpen"]
                .into_iter()
                .find(|id| has(id)),
        )
        .filter(|id| has(id))
        .map(str::to_string)
        .collect();
    let mouth = pushed
        .iter()
        .find(|id| id.contains("Mouth"))
        .cloned()
        .unwrap_or_default();

    let named = |wanted: &[&str], fallback: usize| {
        wanted
            .iter()
            .find(|name| expressions.iter().any(|have| have == *name))
            .map(|name| name.to_string())
            .or_else(|| expressions.get(fallback).cloned())
    };
    let look = |motion: usize, expression: Option<String>| {
        let mut appearance = Appearance::new().motion("Idle", motion.min(idles.max(1) - 1), true);
        if let Some(name) = expression {
            appearance = appearance.expression(name);
        }
        appearance
    };

    let root = std::env::temp_dir().join(format!("vn-live2d-story-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("story")).unwrap();
    fs::create_dir_all(root.join("characters")).unwrap();
    for id in ["one", "two"] {
        std::os::unix::fs::symlink(&model_dir, root.join("characters").join(id)).unwrap();
    }
    fs::write(root.join("story/demo.story"), STORY).unwrap();
    if broken {
        static FALLBACK: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../vn_engine/tests/fixtures/puppet/characters/mary/body.png"
        ));
        let one = root.join("characters/one");
        fs::remove_file(&one).unwrap();
        fs::create_dir_all(&one).unwrap();
        for entry in fs::read_dir(&model_dir).unwrap().filter_map(Result::ok) {
            std::os::unix::fs::symlink(entry.path(), one.join(entry.file_name())).unwrap();
        }
        let moc = one.join(model.replace(".model3.json", ".moc3"));
        let mut bytes = fs::read(&moc).unwrap();
        bytes.truncate(bytes.len() / 2);
        fs::remove_file(&moc).unwrap();
        fs::write(&moc, bytes).unwrap();
        for appearance in ["neutral", "cheerful"] {
            fs::write(one.join(format!("{appearance}.png")), FALLBACK).unwrap();
        }
        println!("Broken run: one's moc3 is truncated, so every instance of it should fall back");
    }

    let neutral = named(&["Normal", "exp_01", "F01"], 0);
    let cheerful = named(&["Smile", "Blushing", "exp_02", "F02"], 1);
    println!("Model: {}/{model}", model_dir.display());
    println!(
        "Idle motions: {idles}, expressions: {} (neutral: {}, cheerful: {})",
        expressions.len(),
        neutral.clone().unwrap_or_else(|| "none".into()),
        cheerful.clone().unwrap_or_else(|| "none".into()),
    );
    println!("Parameters this model has that the probe will push: {pushed:?}");

    let character = |id: &str| {
        Live2dCharacter::new(format!("characters/{id}/{model}"))
            .appearance("neutral", look(0, neutral.clone()))
            .appearance("cheerful", look(4, cheerful.clone()))
    };

    let reloaded = Cell::new(false);
    let pushing = Rc::new(Cell::new(false));
    let start = Rc::clone(&pushing);
    let stop = Rc::clone(&pushing);
    let clock = Cell::new(0.0f32);
    let last = RefCell::new(String::new());
    let released = pushed.clone();
    let finished = Rc::new(Cell::new(false));
    let ending = Rc::clone(&finished);
    let watching = Cell::new(0.0f32);

    VnApp::new("Live2D story")
        .assets(root.clone())
        .schema_file(None)
        .saves_dir(root.join("saves"))
        .initial_screen(ScreenState::Playing)
        .audio(|audio| audio.enabled(false))
        .shader("grain", vn_engine::frame::post::GRAIN)
        .shader("desaturate", vn_engine::frame::post::DESATURATE)
        .character("one", Character::new("Model one"))
        .character("two", Character::new("Model two"))
        .character_visual("one", character("one"))
        .character_visual("two", character("two"))
        .on_scene_enter(|ctx, _| {
            ctx.modes.auto = true;
            None
        })
        .command_as("push", move |_, (): ()| {
            start.set(true);
            println!("Pushing the eyes shut and the mouth open from the frame hook");
            None
        })
        .command_as("release", move |ctx, (): ()| {
            stop.set(false);
            for id in &released {
                ctx.visual_parameter("one", id, None);
            }
            println!("Released them");
            None
        })
        .command_as("snapshot", |ctx, (): ()| {
            ctx.screenshot();
            None
        })
        .command_as("post", |ctx, (): ()| {
            ctx.shader("grain", true);
            ctx.shader_amount("grain", 0.6);
            ctx.shader("desaturate", true);
            ctx.shader_amount("desaturate", 0.8);
            println!("Shader passes on: grain and desaturate");
            None
        })
        .command_as("plain", move |ctx, (): ()| {
            if post {
                return None;
            }
            ctx.shader("grain", false);
            ctx.shader("desaturate", false);
            println!("Shader passes off");
            None
        })
        .command_as("keep", move |ctx, (): ()| {
            match ctx.save(SLOT) {
                Ok(()) => println!("Saved, with whatever is pushed at this line"),
                Err(e) => println!("Save failed: {e}"),
            }
            None
        })
        .command_as("reload", move |ctx, (): ()| {
            if reloaded.replace(true) {
                return None;
            }
            match ctx.load(SLOT) {
                Ok(_) => {
                    println!("Loaded: the models were restarted and the save's push put back");
                    ctx.screenshot();
                }
                Err(e) => println!("Load failed: {e}"),
            }
            None
        })
        .command_as("finish", move |ctx, (): ()| {
            ending.set(true);
            ctx.modes.auto = false;
            println!(
                "The scene finished, auto mode off; Page Up rolls back {} steps",
                ctx.rollback.steps_back()
            );
            smoke.then_some(ScreenState::Quit)
        })
        .on_frame(move |ctx, seconds| {
            let mut shown: Vec<String> = ctx
                .story
                .active_characters()
                .iter()
                .map(|(id, image)| format!("{id}={image}"))
                .collect();
            shown.sort();
            let line = ctx.story.current_say().map_or("", |(_, text)| text);
            let step = format!("[{}] {line}", shown.join(" "));
            if *last.borrow() != step {
                println!("{step}");
                *last.borrow_mut() = step;
            }

            if let Some(every) = watch
                && finished.get()
            {
                let elapsed = watching.get() + seconds;
                watching.set(elapsed);
                if elapsed >= every {
                    watching.set(0.0);
                    println!("Watching: another {every} seconds of motion and physics");
                    ctx.screenshot();
                }
            }

            if !pushing.get() {
                return;
            }
            clock.set(clock.get() + seconds);
            let open = ((clock.get() * 9.0).sin() + 1.0) / 2.0;
            for id in &pushed {
                let value = if *id == mouth { open } else { 0.0 };
                ctx.visual_parameter("one", id, Some(value));
            }
        })
        .run()
}

#[cfg(not(feature = "engine"))]
fn main() {
    eprintln!("Enable the engine feature to run this example");
}
