use std::collections::HashMap;

use raylib::prelude::*;
use vn_core::providers::ResourceProvider;

use crate::providers::raylib_context::RaylibCtx;

#[derive(Default)]
pub struct RaylibTextureManager {
    textures: HashMap<String, Texture2D>,
}

impl ResourceProvider<Texture2D, RaylibCtx> for RaylibTextureManager {
    fn get_texture<'b>(&'b mut self, path: &str, ctx: &mut RaylibCtx) -> &'b Texture2D {
        self.textures.entry(path.to_string()).or_insert_with(|| {
            let mut rl = ctx.rl.borrow_mut();
            rl.load_texture(&ctx.thread, path)
                .expect("Fallback missing")
        })
    }

    fn clear(&mut self) {
        self.textures.clear();
    }
}

impl RaylibTextureManager {
    pub fn get_cached(&self, path: &str) -> Option<&Texture2D> {
        self.textures.get(path)
    }
}
