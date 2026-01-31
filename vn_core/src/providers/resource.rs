use raylib::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct ResourceProvider {
    pub textures: HashMap<String, Texture2D>,
}

impl ResourceProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_texture(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: &str,
    ) -> &Texture2D {
        self.textures.entry(path.to_string()).or_insert_with(|| {
            if std::path::Path::new(path).exists() {
                println!("📥 [CORE] Loading Texture: {}", path);
                rl.load_texture(thread, path)
                    .unwrap_or_else(|_| Self::generate_fallback(rl, thread))
            } else {
                eprintln!("⚠️ [CORE] Texture NOT FOUND: {}. Using fallback.", path);
                Self::generate_fallback(rl, thread)
            }
        })
    }

    fn generate_fallback(rl: &mut RaylibHandle, thread: &RaylibThread) -> Texture2D {
        let image = Image::gen_image_checked(1, 1, 1, 1, Color::MAGENTA, Color::BLACK);
        rl.load_texture_from_image(thread, &image)
            .expect("Critical: Could not generate fallback texture")
    }

    pub fn clear(&mut self) {
        println!("🗑️ [CORE] Clearing VRAM cache...");
        self.textures.clear();
    }
}

impl Drop for ResourceProvider {
    fn drop(&mut self) {
        self.clear();
    }
}
