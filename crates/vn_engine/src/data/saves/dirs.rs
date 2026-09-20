use std::path::PathBuf;

pub fn default_saves_dir(game: &str) -> PathBuf {
    match platform_data_dir() {
        Some(base) => base.join(slug(game)),
        None => PathBuf::from("saves"),
    }
}

fn platform_data_dir() -> Option<PathBuf> {
    let env_dir = |name: &str| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };

    if cfg!(windows) {
        env_dir("APPDATA")
    } else if cfg!(target_os = "macos") {
        env_dir("HOME").map(|home| home.join("Library/Application Support"))
    } else {
        env_dir("XDG_DATA_HOME").or_else(|| env_dir("HOME").map(|home| home.join(".local/share")))
    }
}

pub fn slug(title: &str) -> String {
    let mut slug = String::new();
    for c in title.chars() {
        if c.is_alphanumeric() {
            slug.extend(c.to_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('_') {
            slug.push('_');
        }
    }
    let slug = slug.trim_end_matches('_');
    if slug.is_empty() {
        "game".to_string()
    } else {
        slug.to_string()
    }
}
