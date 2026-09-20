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

const BUILTIN_FONT: &[u8] = include_bytes!("../../assets/fonts/NotoSans-Regular.ttf");
const RASTER_SIZE: i32 = 64;
const GLYPH_RANGES: &[(i32, i32)] = &[
    (0x20, 0x7E),
    (0xA0, 0x17F),
    (0x2010, 0x2027),
    (0x20AC, 0x20AC),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontVariant {
    Bold,
    Italic,
}

pub struct Fonts {
    builtin: Font,
    loaded: HashMap<String, Font>,
    roles: HashMap<FontRole, String>,
    variants: HashMap<(FontRole, FontVariant), String>,
}

impl Fonts {
    pub(crate) fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let builtin = load_font_from_memory(rl, thread, ".ttf", BUILTIN_FONT)
            .expect("Failed to load the built-in font");

        Self {
            builtin,
            loaded: HashMap::new(),
            roles: HashMap::new(),
            variants: HashMap::new(),
        }
    }

    pub(crate) fn load(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        data: io::Result<std::borrow::Cow<'static, [u8]>>,
        source: &str,
        file: &str,
    ) -> bool {
        if self.loaded.contains_key(file) {
            return true;
        }

        let extension = crate::data::assets::extension_of(file);
        match data.and_then(|data| load_font_from_memory(rl, thread, &extension, &data)) {
            Ok(font) => {
                println!("📥 Loaded font: {}", file);
                self.loaded.insert(file.to_string(), font);
                true
            }
            Err(e) => {
                eprintln!("⚠️ Could not load font {}: {}", source, e);
                false
            }
        }
    }

    pub(crate) fn assign(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        data: io::Result<std::borrow::Cow<'static, [u8]>>,
        source: &str,
        role: FontRole,
        file: &str,
    ) {
        if self.load(rl, thread, data, source, file) {
            self.roles.insert(role, file.to_string());
        }
    }

    pub(crate) fn assign_variant(&mut self, role: FontRole, variant: FontVariant, file: &str) {
        if self.loaded.contains_key(file) {
            self.variants.insert((role, variant), file.to_string());
        }
    }

    pub fn styled(&self, role: FontRole, bold: bool, italic: bool) -> (&Font, bool) {
        if italic && let Some(font) = self.variant(role, FontVariant::Italic) {
            return (font, bold);
        }
        if bold && let Some(font) = self.variant(role, FontVariant::Bold) {
            return (font, false);
        }
        (self.get(role), bold)
    }

    fn variant(&self, role: FontRole, variant: FontVariant) -> Option<&Font> {
        self.variants
            .get(&(role, variant))
            .and_then(|file| self.loaded.get(file))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_styled(
        &self,
        d: &mut impl RaylibDraw,
        role: FontRole,
        text: &str,
        position: Vector2,
        size: f32,
        spacing: f32,
        color: Color,
        bold: bool,
        italic: bool,
    ) {
        let (font, faux_bold) = self.styled(role, bold, italic);
        d.draw_text_ex(font, text, position, size, spacing, color);
        if faux_bold {
            let offset = (size / 24.0).max(0.6);
            d.draw_text_ex(
                font,
                text,
                Vector2::new(position.x + offset, position.y),
                size,
                spacing,
                color,
            );
        }
    }

    pub fn measure_styled(
        &self,
        role: FontRole,
        text: &str,
        size: f32,
        spacing: f32,
        bold: bool,
        italic: bool,
    ) -> Vector2 {
        let (font, faux_bold) = self.styled(role, bold, italic);
        let mut measured = font.measure_text(text, size, spacing);
        if faux_bold {
            measured.x += (size / 24.0).max(0.6);
        }
        measured
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

    #[allow(clippy::too_many_arguments)]
    pub fn draw_spaced(
        &self,
        d: &mut impl RaylibDraw,
        role: FontRole,
        text: &str,
        position: Vector2,
        size: f32,
        spacing: f32,
        color: Color,
    ) {
        d.draw_text_ex(self.get(role), text, position, size, spacing, color);
    }

    pub fn measure_spaced(&self, role: FontRole, text: &str, size: f32, spacing: f32) -> Vector2 {
        self.get(role).measure_text(text, size, spacing)
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
