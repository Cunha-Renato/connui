mod atlas;
mod cosmic_types;
mod renderer;

use std::sync::{Arc, Mutex};

use cosmic_text::{FontSystem, SwashCache};

use crate::{renderer::Renderer, types::*};

use atlas::*;
pub use cosmic_types::*;

pub struct Font(Arc<Mutex<FontInner>>);
impl Font {
    #[inline]
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(FontInner {
            system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            atlas_manager: AtlasManager::new(),
        })))
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        Layout::new(self.clone())
    }

    #[inline]
    fn lock<F: FnOnce(&mut FontInner) -> R, R>(&self, f: F) -> R {
        f(&mut self.0.lock().unwrap())
    }
}
impl Default for Font {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
impl Clone for Font {
    #[inline]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

struct FontInner {
    system: FontSystem,
    swash_cache: SwashCache,
    atlas_manager: AtlasManager,
}

/// A shaped, laid-out block of text, ready to measure or draw.
///
/// All positions and sizes returned by this type are in **logical** pixels — i.e. unaffected by display
/// scale factor. Scaling to physical pixels happens internally at draw time,
/// based on `Renderer::scale_factor()`.
pub struct Layout {
    buffer: Buffer,
    font: Font,
}
impl Layout {
    #[inline]
    fn new(font: Font) -> Self {
        let buffer = font.lock(|inner| Buffer::new(&mut inner.system));

        Self { buffer, font }
    }

    #[inline]
    pub fn shape(&mut self) {
        self.font.lock(|inner| self.buffer.shape(&mut inner.system));
    }

    pub fn render<R: Renderer>(&mut self, renderer: &mut R, position: PPoint) {
        self.font.lock(|inner| {
            let mut font_renderer = renderer::FontRenderer::new(inner, renderer);
            font_renderer.render(position, &mut self.buffer);
            font_renderer.finish();
        });
    }
}
impl std::ops::Deref for Layout {
    type Target = Buffer;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
impl std::ops::DerefMut for Layout {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}
