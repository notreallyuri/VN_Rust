use raylib::{RaylibHandle, RaylibThread, texture::Texture2D};
use std::collections::HashMap;

#[derive(Default)]
pub struct ResourceManager {
    pub textures: HashMap<String, Texture2D>,
}

impl ResourceManager {
    pub fn get_or_load(
        &mut self,
        path: &str,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> &Texture2D {
        self.textures.entry(path.to_string()).or_insert_with(|| {
            println!("📥 Loading texture: {}", path);
            rl.load_texture(thread, path)
                .expect("Failed to load texture")
        })
    }
}
