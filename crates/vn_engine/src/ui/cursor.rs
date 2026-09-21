use std::collections::BTreeMap;
use std::ffi::c_void;
use std::os::raw::{c_int, c_uchar};

use raylib::prelude::*;

use crate::input::navigation::InputDevice;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CursorKind {
    #[default]
    Arrow,
    Text,
    Hand,
    Grab,
    Grabbing,
    NotAllowed,
    Custom(&'static str),
    Hidden,
}

impl CursorKind {
    pub fn rank(self) -> u8 {
        match self {
            CursorKind::Arrow => 0,
            CursorKind::Text => 1,
            CursorKind::Hand => 2,
            CursorKind::Custom(_) => 3,
            CursorKind::Grab => 4,
            CursorKind::Grabbing => 5,
            CursorKind::NotAllowed => 6,
            CursorKind::Hidden => 7,
        }
    }

    pub fn stronger(self, other: CursorKind) -> CursorKind {
        match other.rank() >= self.rank() {
            true => other,
            false => self,
        }
    }

    pub fn fallback(self) -> Option<CursorKind> {
        match self {
            CursorKind::Grabbing => Some(CursorKind::Grab),
            CursorKind::Grab | CursorKind::Custom(_) => Some(CursorKind::Hand),
            CursorKind::Hand | CursorKind::Text | CursorKind::NotAllowed => Some(CursorKind::Arrow),
            CursorKind::Arrow | CursorKind::Hidden => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CursorPicture {
    pub file: String,
    pub hotspot: Option<Vector2>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CursorStyle {
    pub pictures: BTreeMap<CursorKind, CursorPicture>,
    pub size: f32,
    pub hotspot: Vector2,
}

impl CursorStyle {
    pub fn new(arrow: impl Into<String>) -> Self {
        Self {
            pictures: BTreeMap::new(),
            size: 32.0,
            hotspot: Vector2::zero(),
        }
        .picture(CursorKind::Arrow, arrow)
    }

    pub fn picture(mut self, kind: CursorKind, file: impl Into<String>) -> Self {
        if kind != CursorKind::Hidden {
            self.pictures.insert(
                kind,
                CursorPicture {
                    file: file.into(),
                    hotspot: None,
                },
            );
        }
        self
    }

    pub fn hand(self, file: impl Into<String>) -> Self {
        self.picture(CursorKind::Hand, file)
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        self
    }

    pub fn hotspot(mut self, x: f32, y: f32) -> Self {
        self.hotspot = unit(x, y);
        self
    }

    pub fn hotspot_for(mut self, kind: CursorKind, x: f32, y: f32) -> Self {
        if let Some(picture) = self.pictures.get_mut(&kind) {
            picture.hotspot = Some(unit(x, y));
        }
        self
    }

    pub fn resolve(&self, kind: CursorKind) -> Option<CursorKind> {
        let mut current = Some(kind);
        while let Some(kind) = current {
            if self.pictures.contains_key(&kind) {
                return Some(kind);
            }
            current = kind.fallback();
        }
        None
    }

    pub fn file(&self, kind: CursorKind) -> Option<&str> {
        let resolved = self.resolve(kind)?;
        Some(self.pictures[&resolved].file.as_str())
    }

    pub fn pixels(&self, kind: CursorKind, natural: Vector2, scale: f32) -> CursorPixels {
        let hotspot = self
            .resolve(kind)
            .and_then(|kind| self.pictures[&kind].hotspot)
            .unwrap_or(self.hotspot);
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        let height = (self.size * scale).round().clamp(MIN_PIXELS, MAX_PIXELS);
        let width = match natural.x > 0.0 && natural.y > 0.0 {
            true => (natural.x / natural.y * height)
                .round()
                .clamp(1.0, MAX_PIXELS),
            false => height,
        };
        CursorPixels {
            width: width as i32,
            height: height as i32,
            hot_x: ((hotspot.x * width).round() as i32).clamp(0, width as i32 - 1),
            hot_y: ((hotspot.y * height).round() as i32).clamp(0, height as i32 - 1),
        }
    }
}

fn unit(x: f32, y: f32) -> Vector2 {
    Vector2::new(x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
}

const MIN_PIXELS: f32 = 8.0;
const MAX_PIXELS: f32 = 256.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorPixels {
    pub width: i32,
    pub height: i32,
    pub hot_x: i32,
    pub hot_y: i32,
}

pub fn path(file: &str) -> String {
    match file.contains('/') {
        true => file.to_lowercase(),
        false => format!("ui/{}", file).to_lowercase(),
    }
}

pub fn effective(
    forced: Option<&(crate::screen::ScreenState, CursorKind)>,
    showing: &crate::screen::ScreenState,
    overlay_open: bool,
    asked: CursorKind,
) -> CursorKind {
    match forced {
        Some((screen, kind)) if screen == showing && !overlay_open => *kind,
        _ => asked,
    }
}

pub fn system_shape(kind: CursorKind) -> Option<MouseCursor> {
    Some(match kind {
        CursorKind::Arrow => MouseCursor::MOUSE_CURSOR_DEFAULT,
        CursorKind::Text => MouseCursor::MOUSE_CURSOR_IBEAM,
        CursorKind::Hand | CursorKind::Grab | CursorKind::Custom(_) => {
            MouseCursor::MOUSE_CURSOR_POINTING_HAND
        }
        CursorKind::Grabbing => MouseCursor::MOUSE_CURSOR_RESIZE_ALL,
        CursorKind::NotAllowed => MouseCursor::MOUSE_CURSOR_NOT_ALLOWED,
        CursorKind::Hidden => return None,
    })
}

#[repr(C)]
struct GlfwImage {
    width: c_int,
    height: c_int,
    pixels: *mut c_uchar,
}

unsafe extern "C" {
    fn glfwCreateCursor(image: *const GlfwImage, xhot: c_int, yhot: c_int) -> *mut c_void;
    fn glfwCreateStandardCursor(shape: c_int) -> *mut c_void;
    fn glfwSetCursor(window: *mut c_void, cursor: *mut c_void);
    fn glfwDestroyCursor(cursor: *mut c_void);
}

const GLFW_SHAPE_BASE: c_int = 0x0003_6000;

enum Source {
    System,
    Pictures {
        style: CursorStyle,
        images: BTreeMap<CursorKind, Image>,
        built_for: Option<i32>,
    },
}

pub(crate) struct Pointer {
    source: Source,
    cursors: BTreeMap<CursorKind, *mut c_void>,
    shown: Option<CursorKind>,
    visible: bool,
}

impl Pointer {
    pub(crate) fn system() -> Self {
        Self {
            source: Source::System,
            cursors: BTreeMap::new(),
            shown: None,
            visible: true,
        }
    }

    pub(crate) fn pictures(
        style: CursorStyle,
        assets: &crate::data::assets::Assets,
    ) -> Result<Self, String> {
        let mut images = BTreeMap::new();
        for (kind, picture) in &style.pictures {
            let path = path(&picture.file);
            let bytes = assets.read(&path).map_err(|e| format!("{}: {}", path, e))?;
            let image =
                Image::load_image_from_mem(&crate::data::assets::extension_of(&path), &bytes)
                    .map_err(|e| format!("{}: {}", path, e))?;
            images.insert(*kind, image);
        }
        Ok(Self {
            source: Source::Pictures {
                style,
                images,
                built_for: None,
            },
            cursors: BTreeMap::new(),
            shown: None,
            visible: true,
        })
    }

    fn picture_cursor(
        style: &CursorStyle,
        kind: CursorKind,
        image: &Image,
        scale: f32,
    ) -> *mut c_void {
        let natural = Vector2::new(image.width as f32, image.height as f32);
        let pixels = style.pixels(kind, natural, scale);
        let mut image = image.clone();
        image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
        image.resize(pixels.width, pixels.height);
        let mut bytes: Vec<u8> = image
            .get_image_data()
            .iter()
            .flat_map(|c| [c.r, c.g, c.b, c.a])
            .collect();
        let glfw = GlfwImage {
            width: pixels.width,
            height: pixels.height,
            pixels: bytes.as_mut_ptr(),
        };
        unsafe { glfwCreateCursor(&glfw, pixels.hot_x, pixels.hot_y) }
    }

    fn rebuild_if_scaled(&mut self) -> bool {
        let Source::Pictures {
            style,
            images,
            built_for,
        } = &mut self.source
        else {
            return false;
        };
        let scale = crate::frame::viewport::current().map_or(1.0, |v| v.scale());
        let height = style
            .pixels(CursorKind::Arrow, Vector2::new(1.0, 1.0), scale)
            .height;
        if *built_for == Some(height) {
            return false;
        }
        let old = std::mem::take(&mut self.cursors);
        for (kind, image) in images.iter() {
            self.cursors
                .insert(*kind, Self::picture_cursor(style, *kind, image, scale));
        }
        *built_for = Some(height);
        self.shown = None;
        for cursor in old.into_values() {
            unsafe { glfwDestroyCursor(cursor) };
        }
        true
    }

    fn cursor_for(&mut self, kind: CursorKind) -> Option<(CursorKind, *mut c_void)> {
        match &self.source {
            Source::Pictures { style, .. } => {
                let key = style.resolve(kind)?;
                self.cursors.get(&key).map(|cursor| (key, *cursor))
            }
            Source::System => {
                let shape = system_shape(kind)?;
                let key = match shape {
                    MouseCursor::MOUSE_CURSOR_IBEAM => CursorKind::Text,
                    MouseCursor::MOUSE_CURSOR_POINTING_HAND => CursorKind::Hand,
                    MouseCursor::MOUSE_CURSOR_RESIZE_ALL => CursorKind::Grabbing,
                    MouseCursor::MOUSE_CURSOR_NOT_ALLOWED => CursorKind::NotAllowed,
                    _ => CursorKind::Arrow,
                };
                if key == CursorKind::Arrow {
                    return Some((key, std::ptr::null_mut()));
                }
                let cursor = *self.cursors.entry(key).or_insert_with(|| unsafe {
                    glfwCreateStandardCursor(GLFW_SHAPE_BASE + shape as c_int)
                });
                Some((key, cursor))
            }
        }
    }

    pub(crate) fn update(&mut self, rl: &mut RaylibHandle, kind: CursorKind, device: InputDevice) {
        let window = unsafe { rl.get_window_handle() };
        let rebuilt = self.rebuild_if_scaled();
        let target = self.cursor_for(kind);

        if let Some((key, cursor)) = target
            && (rebuilt || self.shown != Some(key))
        {
            unsafe { glfwSetCursor(window, cursor) };
            self.shown = Some(key);
        }

        let wanted = device == InputDevice::Mouse && target.is_some();
        if wanted != self.visible {
            match wanted {
                true => rl.show_cursor(),
                false => rl.hide_cursor(),
            }
            self.visible = wanted;
        }
    }
}
