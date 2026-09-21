use raylib::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum CursorKind {
    #[default]
    Arrow,
    Hand,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CursorStyle {
    pub arrow: String,
    pub hand: Option<String>,
    pub size: f32,
    pub hotspot: Vector2,
}

impl CursorStyle {
    pub fn new(arrow: impl Into<String>) -> Self {
        Self {
            arrow: arrow.into(),
            hand: None,
            size: 32.0,
            hotspot: Vector2::zero(),
        }
    }

    pub fn hand(mut self, file: impl Into<String>) -> Self {
        self.hand = Some(file.into());
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        self
    }

    pub fn hotspot(mut self, x: f32, y: f32) -> Self {
        self.hotspot = Vector2::new(x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));
        self
    }

    pub fn file(&self, kind: CursorKind) -> &str {
        match kind {
            CursorKind::Hand => self.hand.as_deref().unwrap_or(&self.arrow),
            CursorKind::Arrow => &self.arrow,
        }
    }

    pub fn pixels(&self, natural: Vector2, scale: f32) -> CursorPixels {
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
            hot_x: ((self.hotspot.x * width).round() as i32).clamp(0, width as i32 - 1),
            hot_y: ((self.hotspot.y * height).round() as i32).clamp(0, height as i32 - 1),
        }
    }
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

#[repr(C)]
struct GlfwImage {
    width: std::os::raw::c_int,
    height: std::os::raw::c_int,
    pixels: *mut std::os::raw::c_uchar,
}

unsafe extern "C" {
    fn glfwCreateCursor(
        image: *const GlfwImage,
        xhot: std::os::raw::c_int,
        yhot: std::os::raw::c_int,
    ) -> *mut std::ffi::c_void;
    fn glfwSetCursor(window: *mut std::ffi::c_void, cursor: *mut std::ffi::c_void);
    fn glfwDestroyCursor(cursor: *mut std::ffi::c_void);
}

pub(crate) struct HardwareCursor {
    style: CursorStyle,
    arrow: Image,
    hand: Option<Image>,
    built: Option<(i32, [*mut std::ffi::c_void; 2])>,
    shown: Option<CursorKind>,
    visible: bool,
}

impl HardwareCursor {
    pub(crate) fn load(
        style: CursorStyle,
        assets: &crate::data::assets::Assets,
    ) -> Result<Self, String> {
        let read = |file: &str| {
            let path = path(file);
            let bytes = assets.read(&path).map_err(|e| format!("{}: {}", path, e))?;
            Image::load_image_from_mem(&crate::data::assets::extension_of(&path), &bytes)
                .map_err(|e| format!("{}: {}", path, e))
        };
        let arrow = read(&style.arrow)?;
        let hand = match &style.hand {
            Some(file) => Some(read(file)?),
            None => None,
        };
        Ok(Self {
            style,
            arrow,
            hand,
            built: None,
            shown: None,
            visible: true,
        })
    }

    fn create(&self, source: &Image, scale: f32) -> *mut std::ffi::c_void {
        let natural = Vector2::new(source.width as f32, source.height as f32);
        let pixels = self.style.pixels(natural, scale);
        let mut image = source.clone();
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

    pub(crate) fn update(
        &mut self,
        rl: &mut RaylibHandle,
        kind: CursorKind,
        device: crate::input::navigation::InputDevice,
    ) {
        let scale = crate::frame::viewport::current().map_or(1.0, |v| v.scale());
        let height = self.style.pixels(Vector2::new(1.0, 1.0), scale).height;

        if self.built.is_none_or(|(built, _)| built != height) {
            let arrow = self.create(&self.arrow, scale);
            let hand = match &self.hand {
                Some(hand) => self.create(hand, scale),
                None => arrow,
            };
            let window = unsafe { rl.get_window_handle() };
            unsafe { glfwSetCursor(window, arrow) };
            if let Some((_, [old_arrow, old_hand])) = self.built.take() {
                unsafe { glfwDestroyCursor(old_arrow) };
                if old_hand != old_arrow {
                    unsafe { glfwDestroyCursor(old_hand) };
                }
            }
            self.built = Some((height, [arrow, hand]));
            self.shown = Some(CursorKind::Arrow);
        }

        if self.shown != Some(kind)
            && let Some((_, cursors)) = self.built
        {
            let cursor = match kind {
                CursorKind::Arrow => cursors[0],
                CursorKind::Hand => cursors[1],
            };
            let window = unsafe { rl.get_window_handle() };
            unsafe { glfwSetCursor(window, cursor) };
            self.shown = Some(kind);
        }

        let wanted = device == crate::input::navigation::InputDevice::Mouse;
        if wanted != self.visible {
            match wanted {
                true => rl.show_cursor(),
                false => rl.hide_cursor(),
            }
            self.visible = wanted;
        }
    }
}
