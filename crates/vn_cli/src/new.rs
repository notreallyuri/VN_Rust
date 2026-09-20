use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

use vn_script::{
    CharacterDef, CommandSig, ParamKind, SCHEMA_FILE_NAME, Schema, SchemaFile, VariableDef,
};

pub const ENGINE_GIT: &str = "https://github.com/notreallyuri/VN_Rust";

pub enum Engine {
    Path(PathBuf),
    Git(String),
}

pub struct Options {
    pub dir: PathBuf,
    pub title: Option<String>,
    pub engine: Option<Engine>,
}

impl Options {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut dir = None;
        let mut title = None;
        let mut engine = None;
        let mut args = args.iter();

        while let Some(arg) = args.next() {
            let mut value = |flag: &str| {
                args.next()
                    .cloned()
                    .ok_or_else(|| format!("{} needs a value", flag))
            };
            match arg.as_str() {
                "--title" => title = Some(value(arg)?),
                "--engine-path" => engine = Some(Engine::Path(value(arg)?.into())),
                "--engine-git" => engine = Some(Engine::Git(value(arg)?)),
                flag if flag.starts_with("--") => return Err(format!("unknown flag {}", flag)),
                path if dir.is_none() => dir = Some(PathBuf::from(path)),
                extra => return Err(format!("unexpected argument '{}'", extra)),
            }
        }

        Ok(Self {
            dir: dir.ok_or("missing the project directory")?,
            title,
            engine,
        })
    }
}

pub fn new(options: Options) -> ExitCode {
    match create(&options) {
        Ok(created) => {
            println!(
                "✅ Created {} ({}) in {}",
                created.title,
                created.package,
                options.dir.display()
            );
            println!();
            println!("  cd {}", options.dir.display());
            println!("  cargo run");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            ExitCode::FAILURE
        }
    }
}

pub struct Created {
    pub package: String,
    pub title: String,
}

pub fn create(options: &Options) -> Result<Created, String> {
    let dir = &options.dir;
    if fs::read_dir(dir).is_ok_and(|mut entries| entries.next().is_some()) {
        return Err(format!("{} already exists and is not empty", dir.display()));
    }

    let name = dir
        .canonicalize()
        .unwrap_or_else(|_| dir.clone())
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .or_else(|| {
            std::env::current_dir()
                .ok()?
                .file_name()?
                .to_str()
                .map(str::to_string)
        })
        .ok_or_else(|| format!("can't name a project after {}", dir.display()))?;
    let package = package_name(&name)
        .ok_or_else(|| format!("'{}' can't be turned into a crate name", name))?;
    let title = options.title.clone().unwrap_or_else(|| title_case(&name));

    let write = |relative: &str, contents: &str| -> Result<(), String> {
        let path = dir.join(relative);
        let io_error = |e: io::Error| format!("{}: {}", path.display(), e);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        fs::write(&path, contents).map_err(io_error)
    };

    fs::create_dir_all(dir).map_err(|e| format!("{}: {}", dir.display(), e))?;
    let engine = engine_dependency(options.engine.as_ref(), dir)?;

    let build = engine.replace("vn_engine", "vn_build");
    write("Cargo.toml", &cargo_toml(&package, &engine, &build))?;
    write("build.rs", BUILD_RS)?;
    write(".gitignore", GITIGNORE)?;
    write("README.md", &readme(&title))?;
    write("src/main.rs", &main_rs(&title))?;
    write("assets/story/start.story", START_STORY)?;
    write("assets/characters/.gitkeep", "")?;
    write("assets/backgrounds/.gitkeep", "")?;
    write("assets/fonts/.gitkeep", "")?;
    write(
        &format!("assets/{}", SCHEMA_FILE_NAME),
        &schema(&title).to_json(),
    )?;

    Ok(Created { package, title })
}

fn engine_dependency(engine: Option<&Engine>, project: &Path) -> Result<String, String> {
    let bundled = Path::new(env!("CARGO_MANIFEST_DIR")).join("../vn_engine");
    let engine = match engine {
        Some(engine) => engine,
        None if bundled.join("Cargo.toml").is_file() => &Engine::Path(bundled),
        None => &Engine::Git(ENGINE_GIT.to_string()),
    };

    match engine {
        Engine::Git(url) => Ok(format!("{{ git = {:?} }}", url)),
        Engine::Path(path) => {
            let engine = path
                .canonicalize()
                .map_err(|e| format!("engine path {}: {}", path.display(), e))?;
            if !engine.join("Cargo.toml").is_file() {
                return Err(format!("{} has no Cargo.toml", engine.display()));
            }
            let project = project
                .canonicalize()
                .map_err(|e| format!("{}: {}", project.display(), e))?;
            if !engine
                .with_file_name("vn_build")
                .join("Cargo.toml")
                .is_file()
            {
                return Err(format!(
                    "{} has no vn_build crate next to it",
                    engine.display()
                ));
            }
            let path = relative_path(&project, &engine).unwrap_or(engine);
            Ok(format!(
                "{{ path = {:?} }}",
                path.to_string_lossy().replace('\\', "/")
            ))
        }
    }
}

fn relative_path(from: &Path, to: &Path) -> Option<PathBuf> {
    let from: Vec<Component> = from.components().collect();
    let to: Vec<Component> = to.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    if common < 2 {
        return None;
    }
    let mut path = PathBuf::new();
    for _ in common..from.len() {
        path.push("..");
    }
    for part in &to[common..] {
        path.push(part);
    }
    Some(path)
}

pub fn package_name(name: &str) -> Option<String> {
    let mut package = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            package.push(c.to_ascii_lowercase());
        } else if !package.is_empty() && !package.ends_with('_') {
            package.push('_');
        }
    }
    let package = package.trim_end_matches('_').to_string();
    match package.chars().next() {
        Some(c) if c.is_ascii_alphabetic() => Some(package),
        _ => None,
    }
}

pub fn title_case(name: &str) -> String {
    name.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            let first = chars.next().map(|c| c.to_uppercase().collect::<String>());
            first.unwrap_or_default() + chars.as_str()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn schema(title: &str) -> SchemaFile {
    let mut schema = Schema::default();
    schema
        .variables
        .insert("player_name".into(), VariableDef::string("Player"));
    schema
        .variables
        .insert("curious".into(), VariableDef::bool(false));
    schema.characters.insert(
        "guide".into(),
        CharacterDef {
            name: "Guide".into(),
            images: vec!["neutral".into(), "happy".into()],
        },
    );
    schema.commands.insert(
        "give_item".into(),
        CommandSig {
            required: vec![ParamKind::Word],
            ..CommandSig::default()
        },
    );
    SchemaFile::new(title, "story", schema)
}

fn cargo_toml(package: &str, engine: &str, build: &str) -> String {
    format!(
        r#"[package]
name = "{package}"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
vn_engine = {engine}

[build-dependencies]
vn_build = {build}

[workspace]
"#
    )
}

const BUILD_RS: &str = r#"fn main() {
    vn_build::Embed::new("assets").exclude(["schema.json"]).run();
}
"#;

const GITIGNORE: &str = "/target/
/saves/
";

fn main_rs(title: &str) -> String {
    MAIN_RS.replace("\"{title}\"", &format!("{:?}", title))
}

const MAIN_RS: &str = r#"use std::process::ExitCode;

use vn_engine::prelude::*;
use vn_engine::raylib::prelude::Color;

const ASSETS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

fn give_item(ctx: &mut GameContext, (item,): (String,)) -> Option<ScreenState> {
    ctx.notify(format!("You got: {}", item.replace('_', " ")));
    None
}

fn main() -> ExitCode {
    let app = VnApp::new("{title}")
        .assets(ASSETS_ROOT)
        .embedded_assets(vn_engine::embedded_assets!())
        .character(
            "guide",
            Character::new("Guide")
                .color(Color::new(150, 190, 230, 255))
                .images(["neutral", "happy"]),
        )
        .variable("player_name", VariableDef::string("Player"))
        .variable("curious", VariableDef::bool(false))
        .command("give_item", give_item);

    match app.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("❌ {}", e);
            ExitCode::FAILURE
        }
    }
}
"#;

const START_STORY: &str = r#"scene start:
  show guide neutral with dissolve
  guide "Welcome! This line comes from assets/story/start.story."
  guide "Characters, variables and commands are registered in src/main.rs."

  choice:
    "Tell me more":
      set curious = true
      show guide happy
      guide "Art goes in assets/characters/guide/happy.png. Until it exists, a placeholder card stands in."
      call give_item notebook
    "Let's get started":
      guide "Straight to it, then."

  jump next_steps

scene next_steps:
  if curious == true:
    guide "You'll find the rest of the syntax in SCRIPT.md."
  guide "Edit this story while the game runs: debug builds reload it."
  "The end."
"#;

fn readme(title: &str) -> String {
    format!(
        r#"# {title}

A visual novel made with [VN_Rust]({ENGINE_GIT}).

```sh
cargo run
```

raylib is built from source on the first run, so you need CMake and a C compiler.

`cargo build --release` puts every file under `assets/` into the executable (see
`build.rs`), so `target/release/<name>` runs anywhere on its own. Debug builds read the
folder instead, so edits to the story show up while the game runs.

| Path | Contents |
|---|---|
| `src/main.rs` | The game: characters, variables, commands and screens, registered on `VnApp` |
| `assets/story/` | `.story` files, all loaded as one story; the game starts at its first scene |
| `assets/characters/<character>/<image>.png` | Character art, for `show <character> <image>` |
| `assets/backgrounds/<image>.png` | Backgrounds, for `background <image>` |
| `assets/fonts/` | Fonts, assigned with `VnApp::font` |
| `assets/schema.json` | The registries, rewritten by debug builds, for `vn check` |

Missing art is drawn as a labeled placeholder, so the story can be played before the
art exists.

Check the story without starting the game:

```sh
vn check assets
```
"#
    )
}
