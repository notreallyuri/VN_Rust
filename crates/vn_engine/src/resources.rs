use raylib::{
    RaylibHandle, RaylibThread,
    color::Color,
    texture::{Image, Texture2D},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{FontRole, Fonts};

const PLACEHOLDER_WIDTH: i32 = 300;
const PLACEHOLDER_HEIGHT: i32 = 500;

pub struct ResourceManager {
    root: PathBuf,
    pub textures: HashMap<String, Texture2D>,
    pub fonts: Fonts,
}

impl ResourceManager {
    pub fn new(root: impl Into<PathBuf>, rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        Self {
            root: root.into(),
            textures: HashMap::new(),
            fonts: Fonts::new(rl, thread),
        }
    }

    pub fn set_font(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        role: FontRole,
        file: &str,
    ) {
        let fonts_dir = self.root.join("fonts");
        self.fonts.assign(rl, thread, &fonts_dir, role, file);
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }

    pub fn get_or_load(
        &mut self,
        path: &str,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> &Texture2D {
        let full_path = self.root.join(path);

        self.textures.entry(path.to_string()).or_insert_with(|| {
            match rl.load_texture(thread, &full_path.to_string_lossy()) {
                Ok(texture) => {
                    println!("📥 Loaded texture: {}", path);
                    texture
                }
                Err(_) => {
                    eprintln!(
                        "⚠️ Could not load {}, using a placeholder",
                        full_path.display()
                    );
                    let image = placeholder_image(path);
                    rl.load_texture_from_image(thread, &image)
                        .expect("Failed to upload placeholder texture")
                }
            }
        })
    }
}

fn placeholder_image(path: &str) -> Image {
    let hash = path.bytes().fold(0x811c9dc5u32, |h, b| {
        (h ^ b as u32).wrapping_mul(0x01000193)
    });
    let [r, g, b, _] = hash.to_le_bytes();
    let fill = Color::new(60 + r % 120, 60 + g % 120, 60 + b % 120, 255);

    let mut image = Image::gen_image_color(PLACEHOLDER_WIDTH, PLACEHOLDER_HEIGHT, fill);
    image.draw_rectangle_lines(
        raylib::math::Rectangle::new(
            0.0,
            0.0,
            PLACEHOLDER_WIDTH as f32,
            PLACEHOLDER_HEIGHT as f32,
        ),
        4,
        Color::RAYWHITE,
    );

    let label = path.strip_suffix(".png").unwrap_or(path);
    for (i, part) in label.split('/').enumerate() {
        image.draw_text(part, 16, 16 + i as i32 * 28, 20, Color::RAYWHITE);
    }

    image
}
