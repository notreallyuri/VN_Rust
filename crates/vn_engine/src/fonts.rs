use raylib::{
    RaylibHandle, RaylibThread,
    color::Color,
    consts::TextureFilter,
    ffi,
    math::Vector2,
    prelude::{RaylibDraw, RaylibFont},
    text::Font,
};
use std::collections::HashMap;
use std::ffi::CString;
use std::io;
use std::path::Path;

const BUILTIN_FONT: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
const RASTER_SIZE: i32 = 64;
const GLYPH_RANGES: &[(i32, i32)] = &[
    (0x20, 0x7E),
    (0xA0, 0x17F),
    (0x2010, 0x2027),
    (0x20AC, 0x20AC),
    (0x2190, 0x2193),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontRole {
    Default,
    Title,
    Menu,
    Button,
    Dialogue,
    Speaker,
    Choice,
    Custom(&'static str),
}

pub struct Fonts {
    builtin: Font,
    loaded: HashMap<String, Font>,
    roles: HashMap<FontRole, String>,
}

impl Fonts {
    pub(crate) fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let builtin = load_font_from_memory(rl, thread, ".ttf", BUILTIN_FONT)
            .expect("Failed to load the built-in font");

        Self {
            builtin,
            loaded: HashMap::new(),
            roles: HashMap::new(),
        }
    }

    pub(crate) fn assign(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        fonts_dir: &Path,
        role: FontRole,
        file: &str,
    ) {
        if !self.loaded.contains_key(file) {
            let full_path = fonts_dir.join(file);

            match load_font_file(rl, thread, &full_path) {
                Ok(font) => {
                    println!("📥 Loaded font: {}", file);
                    self.loaded.insert(file.to_string(), font);
                }
                Err(e) => {
                    eprintln!("⚠️ Could not load font {}: {}", full_path.display(), e);
                    return;
                }
            }
        }

        self.roles.insert(role, file.to_string());
    }

    pub fn get(&self, role: FontRole) -> &Font {
        self.lookup(role)
            .or_else(|| self.lookup(FontRole::Default))
            .unwrap_or(&self.builtin)
    }

    fn lookup(&self, role: FontRole) -> Option<&Font> {
        self.roles.get(&role).and_then(|file| self.loaded.get(file))
    }

    pub fn draw(
        &self,
        d: &mut impl RaylibDraw,
        role: FontRole,
        text: &str,
        position: Vector2,
        size: f32,
        color: Color,
    ) {
        d.draw_text_ex(self.get(role), text, position, size, 0.0, color);
    }

    pub fn measure(&self, role: FontRole, text: &str, size: f32) -> Vector2 {
        self.get(role).measure_text(text, size, 0.0)
    }

    pub fn wrap(&self, role: FontRole, text: &str, size: f32, max_width: f32) -> Vec<String> {
        let mut lines = Vec::new();

        for paragraph in text.split('\n') {
            let mut line = String::new();

            for word in paragraph.split_whitespace() {
                let candidate = if line.is_empty() {
                    word.to_string()
                } else {
                    format!("{} {}", line, word)
                };

                if line.is_empty() || self.measure(role, &candidate, size).x <= max_width {
                    line = candidate;
                } else {
                    lines.push(std::mem::replace(&mut line, word.to_string()));
                }
            }

            lines.push(line);
        }

        lines
    }
}

fn load_font_file(rl: &mut RaylibHandle, thread: &RaylibThread, path: &Path) -> io::Result<Font> {
    let data = std::fs::read(path)?;
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_lowercase()))
        .unwrap_or_default();

    load_font_from_memory(rl, thread, &extension, &data)
}

fn load_font_from_memory(
    _: &mut RaylibHandle,
    _: &RaylibThread,
    extension: &str,
    data: &[u8],
) -> io::Result<Font> {
    let invalid = |msg: &str| io::Error::new(io::ErrorKind::InvalidData, msg.to_string());

    let file_type = CString::new(extension).map_err(|_| invalid("bad file extension"))?;
    let mut codepoints: Vec<i32> = GLYPH_RANGES.iter().flat_map(|&(a, b)| a..=b).collect();

    let raw = unsafe {
        ffi::LoadFontFromMemory(
            file_type.as_ptr(),
            data.as_ptr(),
            data.len() as i32,
            RASTER_SIZE,
            codepoints.as_mut_ptr(),
            codepoints.len() as i32,
        )
    };

    if raw.glyphs.is_null() || raw.texture.id == 0 {
        return Err(invalid(
            "not a font raylib can read (expected .ttf or .otf)",
        ));
    }

    let mut font = unsafe { Font::from_raw(raw) };

    let raw: &mut ffi::Font = font.as_mut();
    unsafe {
        ffi::GenTextureMipmaps(&mut raw.texture);
        ffi::SetTextureFilter(raw.texture, TextureFilter::TEXTURE_FILTER_TRILINEAR as i32);
    }

    Ok(font)
}
