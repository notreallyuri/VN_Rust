use std::borrow::Cow;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub type EmbeddedFile = (&'static str, &'static [u8]);

#[macro_export]
macro_rules! embedded_assets {
    () => {
        include!(concat!(env!("OUT_DIR"), "/vn_assets.rs"))
    };
}

#[derive(Clone)]
pub enum Assets {
    Dir(PathBuf),
    Embedded(&'static [EmbeddedFile]),
}

impl fmt::Debug for Assets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Assets::Dir(dir) => write!(f, "Assets::Dir({})", dir.display()),
            Assets::Embedded(files) => write!(f, "Assets::Embedded({} files)", files.len()),
        }
    }
}

impl From<PathBuf> for Assets {
    fn from(dir: PathBuf) -> Self {
        Assets::Dir(dir)
    }
}

impl From<&Path> for Assets {
    fn from(dir: &Path) -> Self {
        Assets::Dir(dir.to_path_buf())
    }
}

impl From<&str> for Assets {
    fn from(dir: &str) -> Self {
        Assets::Dir(PathBuf::from(dir))
    }
}

impl From<String> for Assets {
    fn from(dir: String) -> Self {
        Assets::Dir(PathBuf::from(dir))
    }
}

fn normalize(relative: &str) -> String {
    relative
        .replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>()
        .join("/")
}

impl Assets {
    pub fn dir(&self) -> Option<&Path> {
        match self {
            Assets::Dir(dir) => Some(dir),
            Assets::Embedded(_) => None,
        }
    }

    pub fn is_embedded(&self) -> bool {
        matches!(self, Assets::Embedded(_))
    }

    pub fn describe(&self, relative: &str) -> String {
        match self {
            Assets::Dir(dir) => dir.join(normalize(relative)).display().to_string(),
            Assets::Embedded(_) => format!("<embedded>/{}", normalize(relative)),
        }
    }

    pub fn read(&self, relative: &str) -> io::Result<Cow<'static, [u8]>> {
        let relative = normalize(relative);
        match self {
            Assets::Dir(dir) => fs::read(dir.join(&relative)).map(Cow::Owned),
            Assets::Embedded(files) => files
                .iter()
                .find(|(path, _)| *path == relative)
                .map(|(_, bytes)| Cow::Borrowed(*bytes))
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("{} is not among the embedded assets", relative),
                    )
                }),
        }
    }

    pub fn read_to_string(&self, relative: &str) -> io::Result<String> {
        String::from_utf8(self.read(relative)?.into_owned())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn exists(&self, relative: &str) -> bool {
        let relative = normalize(relative);
        match self {
            Assets::Dir(dir) => dir.join(&relative).is_file(),
            Assets::Embedded(files) => files.iter().any(|(path, _)| *path == relative),
        }
    }

    pub fn files_under(&self, dir: &str) -> io::Result<Vec<String>> {
        let dir = normalize(dir);
        let prefix = if dir.is_empty() {
            String::new()
        } else {
            format!("{}/", dir)
        };
        let mut files: Vec<String> = match self {
            Assets::Dir(root) => {
                let mut found = Vec::new();
                collect(root, &root.join(&dir), &mut found)?;
                found
            }
            Assets::Embedded(files) => files
                .iter()
                .map(|(path, _)| path.to_string())
                .filter(|path| path.starts_with(&prefix))
                .collect(),
        };
        files.sort();
        Ok(files)
    }
}

fn collect(root: &Path, dir: &Path, files: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(root, &path, files)?;
        } else if let Ok(relative) = path.strip_prefix(root) {
            files.push(normalize(&relative.to_string_lossy()));
        }
    }
    Ok(())
}

pub fn extension_of(relative: &str) -> String {
    Path::new(relative)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_lowercase()))
        .unwrap_or_default()
}
