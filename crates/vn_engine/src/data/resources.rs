use crate::data::assets::{Assets, extension_of};
use crate::ui::fonts::{FontRole, FontVariant, Fonts};
use raylib::{
    RaylibHandle, RaylibThread,
    color::Color,
    texture::{Image, Texture2D},
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

const PLACEHOLDER_WIDTH: i32 = 300;
const PLACEHOLDER_HEIGHT: i32 = 500;
const BACKGROUND_PLACEHOLDER_WIDTH: i32 = 1280;
const BACKGROUND_PLACEHOLDER_HEIGHT: i32 = 720;

pub fn character_path(character: &str, image: &str) -> String {
    format!("characters/{}/{}.png", character, image).to_lowercase()
}

pub fn background_path(image: &str) -> String {
    format!("backgrounds/{}.png", image).to_lowercase()
}

pub struct ResourceManager {
    assets: Assets,
    pub textures: HashMap<String, Texture2D>,
    pub fonts: Fonts,
    requested: RefCell<HashSet<String>>,
}

impl ResourceManager {
    pub fn new(assets: impl Into<Assets>, rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        Self {
            assets: assets.into(),
            textures: HashMap::new(),
            fonts: Fonts::new(rl, thread),
            requested: RefCell::new(HashSet::new()),
        }
    }

    pub fn set_font(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        role: FontRole,
        file: &str,
    ) {
        let path = format!("fonts/{}", file);
        let data = self.assets.read(&path);
        let source = self.assets.describe(&path);
        self.fonts.assign(rl, thread, data, &source, role, file);
    }

    pub fn set_font_variant(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        role: FontRole,
        variant: FontVariant,
        file: &str,
    ) {
        let path = format!("fonts/{}", file);
        let data = self.assets.read(&path);
        let source = self.assets.describe(&path);
        if self.fonts.load(rl, thread, data, &source, file) {
            self.fonts.assign_variant(role, variant, file);
        }
    }

    pub fn assets(&self) -> &Assets {
        &self.assets
    }

    pub fn texture(&self, path: &str) -> Option<&Texture2D> {
        let texture = self.textures.get(path);
        if texture.is_none() {
            self.requested.borrow_mut().insert(path.to_string());
        }
        texture
    }

    pub fn load_requested(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let requested: Vec<String> = self.requested.borrow_mut().drain().collect();
        for path in requested {
            self.get_or_load(&path, rl, thread);
        }
    }

    pub fn get_or_load(
        &mut self,
        path: &str,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> &Texture2D {
        let assets = &self.assets;

        self.textures.entry(path.to_string()).or_insert_with(|| {
            let loaded = assets
                .read(path)
                .map_err(|e| e.to_string())
                .and_then(|bytes| {
                    Image::load_image_from_mem(&extension_of(path), &bytes)
                        .map_err(|e| e.to_string())
                })
                .and_then(|image| {
                    rl.load_texture_from_image(thread, &image)
                        .map_err(|e| e.to_string())
                });
            match loaded {
                Ok(texture) => {
                    println!("📥 Loaded texture: {}", path);
                    texture
                }
                Err(_) => {
                    eprintln!(
                        "⚠️ Could not load {}, using a placeholder",
                        assets.describe(path)
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
    if path.starts_with("backgrounds/") {
        return background_placeholder(path);
    }

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

fn background_placeholder(path: &str) -> Image {
    let mut image = Image::gen_image_color(
        BACKGROUND_PLACEHOLDER_WIDTH,
        BACKGROUND_PLACEHOLDER_HEIGHT,
        Color::new(28, 26, 36, 255),
    );
    let label = path.strip_suffix(".png").unwrap_or(path);
    image.draw_text(label, 24, 24, 28, Color::new(120, 116, 140, 255));
    image
}
