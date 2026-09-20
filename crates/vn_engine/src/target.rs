use raylib::prelude::*;

#[derive(Default)]
pub struct RenderTarget {
    frame: Option<RenderTexture2D>,
    size: (i32, i32),
}

impl RenderTarget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resize(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, size: (i32, i32)) {
        if self.size == size && self.frame.is_some() {
            return;
        }
        if size.0 <= 0 || size.1 <= 0 {
            self.frame = None;
            self.size = size;
            return;
        }

        self.size = size;
        self.frame = match rl.load_render_texture(thread, size.0 as u32, size.1 as u32) {
            Ok(frame) => {
                frame
                    .texture()
                    .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
                Some(frame)
            }
            Err(e) => {
                eprintln!("⚠️ Drawing straight to the screen, no render target: {}", e);
                None
            }
        };
    }

    pub fn frame(&self) -> Option<&RenderTexture2D> {
        self.frame.as_ref()
    }

    pub fn frame_mut(&mut self) -> Option<&mut RenderTexture2D> {
        self.frame.as_mut()
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    pub fn source(&self) -> Rectangle {
        Rectangle::new(0.0, 0.0, self.size.0 as f32, -(self.size.1 as f32))
    }
}

pub fn destination(target: (i32, i32), screen: (i32, i32)) -> Rectangle {
    let whole = Rectangle::new(0.0, 0.0, screen.0 as f32, screen.1 as f32);
    if target.0 <= 0 || target.1 <= 0 {
        return whole;
    }

    let scale = f32::min(
        screen.0 as f32 / target.0 as f32,
        screen.1 as f32 / target.1 as f32,
    );
    let width = (target.0 as f32 * scale).round();
    let height = (target.1 as f32 * scale).round();
    Rectangle::new(
        ((screen.0 as f32 - width) / 2.0).round(),
        ((screen.1 as f32 - height) / 2.0).round(),
        width,
        height,
    )
}

pub fn copy_into(
    d: &mut RaylibDrawHandle,
    thread: &RaylibThread,
    from: &RenderTexture2D,
    into: &mut RenderTexture2D,
    size: (i32, i32),
) {
    let whole = Rectangle::new(0.0, 0.0, size.0 as f32, size.1 as f32);
    let flipped = Rectangle::new(0.0, 0.0, size.0 as f32, -(size.1 as f32));
    let mut t = d.begin_texture_mode(thread, into);
    t.draw_texture_pro(
        from.texture(),
        flipped,
        whole,
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );
}
