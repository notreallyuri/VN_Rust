use std::cell::RefCell;
use std::ffi::{CString, c_char, c_void};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use vn_engine::raylib::{self, prelude::*};

use crate::{Error, ModelAssets};

const ERROR_CAPACITY: usize = 1024;
static ACTIVE: AtomicBool = AtomicBool::new(false);
thread_local! {
    static SHARED: RefCell<Weak<Runtime>> = const { RefCell::new(Weak::new()) };
}

unsafe extern "C" {
    fn vn_cubism_start(error: *mut c_char, capacity: usize) -> i32;
    fn vn_cubism_stop();
    fn vn_model_create(
        manifest: *const u8,
        manifest_size: i32,
        moc: *const u8,
        moc_size: i32,
        textures: i32,
        error: *mut c_char,
        capacity: usize,
    ) -> *mut c_void;
    fn vn_model_destroy(model: *mut c_void);
    fn vn_model_size(
        model: *mut c_void,
        width: *mut f32,
        height: *mut f32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn vn_model_asset(
        model: *mut c_void,
        kind: i32,
        name: *const c_char,
        index: i32,
        bytes: *const u8,
        size: i32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn vn_model_texture(
        model: *mut c_void,
        slot: u32,
        texture: u32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn vn_model_motion(
        model: *mut c_void,
        group: *const c_char,
        index: i32,
        looping: i32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn vn_model_expression(
        model: *mut c_void,
        name: *const c_char,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn vn_model_update(
        model: *mut c_void,
        seconds: f32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn vn_model_parameter(
        model: *mut c_void,
        name: *const c_char,
        value: f32,
        clear: i32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn vn_model_draw(
        model: *mut c_void,
        matrix: *const f32,
        opacity: f32,
        width: u32,
        height: u32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
}

fn error(buffer: &[c_char]) -> Error {
    let bytes: Vec<u8> = buffer
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    Error(String::from_utf8_lossy(&bytes).into_owned())
}

fn call(operation: impl FnOnce(*mut c_char, usize) -> i32) -> Result<(), Error> {
    let mut buffer = [0; ERROR_CAPACITY];
    if operation(buffer.as_mut_ptr(), buffer.len()) == 0 {
        Err(error(&buffer))
    } else {
        Ok(())
    }
}

fn name(value: &str) -> Result<CString, Error> {
    CString::new(value).map_err(|_| Error("Cubism names cannot contain NUL".into()))
}

pub struct Cubism {
    runtime: Rc<Runtime>,
}

struct Runtime {
    _thread_bound: PhantomData<Rc<()>>,
}

pub fn with_cubism<R>(
    rl: &mut RaylibHandle,
    _thread: &RaylibThread,
    operation: impl FnOnce(&Cubism, &mut RaylibHandle) -> Result<R, Error>,
) -> Result<R, Error> {
    let sdk = Cubism {
        runtime: acquire(rl, _thread)?,
    };
    operation(&sdk, rl)
}

fn acquire(_rl: &mut RaylibHandle, _thread: &RaylibThread) -> Result<Rc<Runtime>, Error> {
    if let Some(runtime) = SHARED.with(|shared| shared.borrow().upgrade()) {
        return Ok(runtime);
    }
    if ACTIVE
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err(Error("a Cubism runtime is active on another thread".into()));
    }
    if let Err(error) = call(|error, capacity| unsafe { vn_cubism_start(error, capacity) }) {
        ACTIVE.store(false, Ordering::Release);
        return Err(error);
    }
    let runtime = Rc::new(Runtime {
        _thread_bound: PhantomData,
    });
    SHARED.with(|shared| *shared.borrow_mut() = Rc::downgrade(&runtime));
    Ok(runtime)
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe { vn_cubism_stop() };
        ACTIVE.store(false, Ordering::Release);
    }
}

pub struct Model<'sdk> {
    handle: NonNull<c_void>,
    // Native renderer borrows texture ids; keep textures until after its drop.
    textures: Vec<Texture2D>,
    _runtime: Rc<Runtime>,
    _scope: PhantomData<&'sdk Cubism>,
}

impl<'sdk> Model<'sdk> {
    pub fn canvas_size(&self) -> Result<Vector2, Error> {
        let (mut width, mut height) = (0.0, 0.0);
        call(|error, capacity| unsafe {
            vn_model_size(
                self.handle.as_ptr(),
                &mut width,
                &mut height,
                error,
                capacity,
            )
        })?;
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return Err(Error("model returned invalid canvas dimensions".into()));
        }
        Ok(Vector2::new(width, height))
    }

    pub fn load(
        sdk: &'sdk Cubism,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        assets: &ModelAssets,
    ) -> Result<Self, Error> {
        Self::load_in(Rc::clone(&sdk.runtime), rl, thread, assets)
    }

    fn load_in(
        runtime: Rc<Runtime>,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        assets: &ModelAssets,
    ) -> Result<Self, Error> {
        let refs = &assets.settings.file_references;
        let moc = &assets.files[&refs.moc];
        let texture_count =
            i32::try_from(refs.textures.len()).map_err(|_| Error("too many textures".into()))?;
        let mut buffer = [0; ERROR_CAPACITY];
        // SAFETY: ModelAssets checked nonempty buffers and i32 lengths. Native
        // code copies model data; it never retains the borrowed Rust buffers.
        let handle = unsafe {
            vn_model_create(
                assets.manifest.as_ptr(),
                assets.manifest.len() as i32,
                moc.as_ptr(),
                moc.len() as i32,
                texture_count,
                buffer.as_mut_ptr(),
                buffer.len(),
            )
        };
        let handle = NonNull::new(handle).ok_or_else(|| error(&buffer))?;
        let mut model = Self {
            handle,
            textures: Vec::new(),
            _runtime: runtime,
            _scope: PhantomData,
        };
        for (slot, path) in refs.textures.iter().enumerate() {
            let extension = std::path::Path::new(path)
                .extension()
                .and_then(|s| s.to_str())
                .ok_or_else(|| Error(format!("texture has no extension: {path}")))?;
            let image = Image::load_image_from_mem(&format!(".{extension}"), &assets.files[path])
                .map_err(|e| Error(format!("{path}: {e}")))?;
            let texture = rl
                .load_texture_from_image(thread, &image)
                .map_err(|e| Error(format!("{path}: {e}")))?;
            texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            call(|error, capacity| unsafe {
                vn_model_texture(handle.as_ptr(), slot as u32, texture.id, error, capacity)
            })?;
            model.textures.push(texture);
        }
        if let Some(path) = &refs.physics {
            model.asset(0, "", 0, &assets.files[path])?;
        }
        if let Some(path) = &refs.pose {
            model.asset(1, "", 0, &assets.files[path])?;
        }
        for (group, motions) in &refs.motions {
            for (index, motion) in motions.iter().enumerate() {
                let index = i32::try_from(index).map_err(|_| Error("too many motions".into()))?;
                model.asset(2, group, index, &assets.files[&motion.file])?;
            }
        }
        for expression in &refs.expressions {
            model.asset(3, &expression.name, 0, &assets.files[&expression.file])?;
        }
        Ok(model)
    }

    fn asset(&mut self, kind: i32, id: &str, index: i32, bytes: &[u8]) -> Result<(), Error> {
        let id = name(id)?;
        // SAFETY: live model; buffer is borrowed only for this call.
        call(|error, capacity| unsafe {
            vn_model_asset(
                self.handle.as_ptr(),
                kind,
                id.as_ptr(),
                index,
                bytes.as_ptr(),
                bytes.len() as i32,
                error,
                capacity,
            )
        })
    }

    pub fn start_motion(&mut self, group: &str, index: usize, looping: bool) -> Result<(), Error> {
        let group = name(group)?;
        let index = i32::try_from(index).map_err(|_| Error("motion index too large".into()))?;
        // SAFETY: live model and NUL-terminated group, not retained by the bridge.
        call(|error, capacity| unsafe {
            vn_model_motion(
                self.handle.as_ptr(),
                group.as_ptr(),
                index,
                i32::from(looping),
                error,
                capacity,
            )
        })
    }

    pub fn set_expression(&mut self, expression: &str) -> Result<(), Error> {
        let expression = name(expression)?;
        // SAFETY: live model and borrowed NUL-terminated name.
        call(|error, capacity| unsafe {
            vn_model_expression(self.handle.as_ptr(), expression.as_ptr(), error, capacity)
        })
    }

    pub fn update(&mut self, seconds: f32) -> Result<(), Error> {
        if !seconds.is_finite() || seconds < 0.0 {
            return Err(Error(
                "animation delta must be finite and nonnegative".into(),
            ));
        }
        // Avoid exploding physics after a breakpoint or a suspended window.
        call(|error, capacity| unsafe {
            vn_model_update(self.handle.as_ptr(), seconds.min(0.1), error, capacity)
        })
    }

    pub fn set_parameter(&mut self, id: &str, value: Option<f32>) -> Result<(), Error> {
        if value.is_some_and(|v| !v.is_finite()) {
            return Err(Error("parameter value must be finite".into()));
        }
        let id = name(id)?;
        // SAFETY: live model and borrowed NUL-terminated name.
        call(|error, capacity| unsafe {
            vn_model_parameter(
                self.handle.as_ptr(),
                id.as_ptr(),
                value.unwrap_or(0.0),
                i32::from(value.is_none()),
                error,
                capacity,
            )
        })
    }

    pub fn draw(
        &mut self,
        _draw: &mut impl RaylibDraw,
        matrix: [f32; 16],
        size: (u32, u32),
        opacity: f32,
    ) -> Result<(), Error> {
        if size.0 == 0
            || size.1 == 0
            || !opacity.is_finite()
            || !matrix.iter().all(|v| v.is_finite())
        {
            return Err(Error(
                "invalid Cubism drawing dimensions, matrix, or opacity".into(),
            ));
        }
        // SAFETY: draw scope is active, model belongs to this SDK thread. Flush
        // queued raylib geometry before the native renderer changes GL state.
        unsafe { raylib::ffi::rlDrawRenderBatchActive() };
        call(|error, capacity| unsafe {
            vn_model_draw(
                self.handle.as_ptr(),
                matrix.as_ptr(),
                opacity.clamp(0.0, 1.0),
                size.0,
                size.1,
                error,
                capacity,
            )
        })
    }
}

impl Drop for Model<'_> {
    fn drop(&mut self) {
        // SAFETY: unique native ownership; SDK/window and textures are still alive.
        unsafe { vn_model_destroy(self.handle.as_ptr()) };
    }
}

#[cfg(feature = "engine")]
impl Model<'static> {
    pub(crate) fn load_owned(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        assets: &ModelAssets,
    ) -> Result<Self, Error> {
        Self::load_in(acquire(rl, thread)?, rl, thread, assets)
    }
}
