use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const OUTPUT_FILE: &str = "vn_assets.rs";
pub const ENV_OVERRIDE: &str = "VN_EMBED_ASSETS";

pub fn embed_assets(dir: impl Into<PathBuf>) {
    Embed::new(dir).run();
}

#[derive(Clone, Debug)]
pub struct Embed {
    dir: PathBuf,
    exclude: Vec<String>,
    always: bool,
}

impl Embed {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            exclude: Vec::new(),
            always: false,
        }
    }

    pub fn exclude<S: Into<String>>(mut self, paths: impl IntoIterator<Item = S>) -> Self {
        self.exclude.extend(paths.into_iter().map(Into::into));
        self
    }

    pub fn always(mut self, always: bool) -> Self {
        self.always = always;
        self
    }

    pub fn run(self) {
        let manifest = env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from);
        let dir = match &manifest {
            Some(manifest) if self.dir.is_relative() => manifest.join(&self.dir),
            _ => self.dir.clone(),
        };
        let out =
            PathBuf::from(env::var_os("OUT_DIR").expect("novn_build runs from a build script"));

        println!("cargo:rerun-if-changed={}", dir.display());
        println!("cargo:rerun-if-env-changed={}", ENV_OVERRIDE);

        let release = env::var("PROFILE").is_ok_and(|profile| profile == "release");
        let embed = match env::var(ENV_OVERRIDE).as_deref() {
            Ok("1") | Ok("true") => true,
            Ok("0") | Ok("false") => false,
            _ => self.always || release,
        };

        let files = if embed {
            self.files(&dir).unwrap_or_else(|e| {
                panic!(
                    "novn_build: could not read the assets in {}: {}",
                    dir.display(),
                    e
                )
            })
        } else {
            Vec::new()
        };

        let source = generate(&files);
        let path = out.join(OUTPUT_FILE);
        if fs::read_to_string(&path).ok().as_deref() != Some(source.as_str()) {
            fs::write(&path, source).expect("novn_build: could not write the embedded asset list");
        }
    }

    pub fn files(&self, dir: &Path) -> io::Result<Vec<(String, PathBuf)>> {
        let mut files = Vec::new();
        collect(dir, dir, &mut files)?;
        files.retain(|(relative, _)| {
            !self.exclude.iter().any(|excluded| {
                relative == excluded || relative.starts_with(&format!("{}/", excluded))
            })
        });
        files.sort();
        Ok(files)
    }
}

fn collect(root: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let hidden = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('.'));
        if hidden {
            continue;
        }
        if path.is_dir() {
            collect(root, &path, files)?;
        } else if let Ok(relative) = path.strip_prefix(root) {
            let relative = relative
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let absolute = path.canonicalize().unwrap_or(path);
            files.push((relative, absolute));
        }
    }
    Ok(())
}

pub fn generate(files: &[(String, PathBuf)]) -> String {
    let mut source = String::from("{\n    static VN_ASSETS: &[(&str, &[u8])] = &[\n");
    for (relative, absolute) in files {
        source.push_str(&format!(
            "        ({:?}, include_bytes!({:?}) as &[u8]),\n",
            relative,
            absolute.to_string_lossy()
        ));
    }
    source.push_str("    ];\n    VN_ASSETS\n}\n");
    source
}
