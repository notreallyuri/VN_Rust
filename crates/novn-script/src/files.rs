use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const STORY_EXTENSION: &str = "story";

pub fn story_files(dir: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let dir = dir.as_ref();
    let mut files = Vec::new();
    collect(dir, &mut files)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", dir.display(), e)))?;
    files.sort();
    Ok(files)
}

fn collect(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == STORY_EXTENSION) {
            files.push(path);
        }
    }
    Ok(())
}

pub fn read_sources(paths: &[PathBuf]) -> io::Result<Vec<(String, String)>> {
    paths
        .iter()
        .map(|path| {
            fs::read_to_string(path)
                .map(|source| (path.display().to_string(), source))
                .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", path.display(), e)))
        })
        .collect()
}
