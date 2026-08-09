use std::collections::HashMap;

use cosmic_text::{CacheKey, SwashImage};
use etagere::{Allocation, AtlasAllocator, euclid::Size2D};

use crate::types::*;

pub const ATLAS_WIDTH: usize = 1024;
pub const ATLAS_HEIGHT: usize = 1024;

pub struct AtlasManager {
    glyphs: HashMap<CacheKey, (usize, CachedGlyph)>,
    atlases: Vec<Atlas>,
    uploads: Vec<PendingUpload>,
}
impl AtlasManager {
    #[inline]
    pub fn new() -> Self {
        Self {
            atlases: vec![Atlas::new(Id::new("connui_font::atlas::0"))],
            glyphs: HashMap::default(),
            uploads: Vec::new(),
        }
    }

    pub fn get_or_maybe_insert<F>(&mut self, key: CacheKey, f: F) -> Option<&(usize, CachedGlyph)>
    where
        F: FnOnce() -> Option<SwashImage>,
    {
        use std::collections::hash_map::Entry;

        match self.glyphs.entry(key) {
            Entry::Occupied(entry) => Some(&*entry.into_mut()),
            Entry::Vacant(entry) => {
                let image = f()?;
                let size = Size2D::new(image.placement.width as i32, image.placement.height as i32);

                if size.width <= 0 || size.height <= 0 {
                    return None;
                }

                if size.width as usize > ATLAS_WIDTH || size.height as usize > ATLAS_HEIGHT {
                    panic!(
                        "Glyph size {}x{} exceeds atlas size {ATLAS_WIDTH}x{ATLAS_HEIGHT}",
                        size.width, size.height
                    );
                }

                let allocation = self
                    .atlases
                    .last_mut()
                    .and_then(|atlas| atlas.allocator.allocate(size))
                    .unwrap_or_else(|| {
                        let id = Id::new(format!("connui_font::Atlas::{}", self.atlases.len()));
                        let mut atlas = Atlas::new(id);

                        // SAFETY: Should never fail.
                        let allocation = atlas.allocator
                            .allocate(size)
                            .expect("fresh atlas has room for a glyph that already passed the atlas-bounds check");

                        self.atlases.push(atlas);

                        allocation
                    });

                let cached = CachedGlyph::new(allocation, &image);
                let atlas_idx = self.atlases.len() - 1;

                self.uploads.push(PendingUpload {
                    atlas: atlas_idx,
                    allocation,
                    image,
                });

                Some(&*entry.insert((atlas_idx, cached)))
            }
        }
    }

    #[inline]
    pub fn atlas(&self, index: usize) -> &Atlas {
        &self.atlases[index]
    }

    #[inline]
    pub fn take_uploads(&mut self) -> Vec<PendingUpload> {
        std::mem::take(&mut self.uploads)
    }
}
impl Default for AtlasManager {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

pub struct Atlas {
    pub allocator: AtlasAllocator,
    pub id: Id,
}
impl Atlas {
    fn new(id: Id) -> Self {
        Self {
            id,
            allocator: AtlasAllocator::new(Size2D::new(ATLAS_WIDTH as i32, ATLAS_HEIGHT as i32)),
        }
    }
}

#[derive(Clone)]
pub struct CachedGlyph {
    pub rect: PRect,
    pub uv: Rect<f32, f32>,
}
impl CachedGlyph {
    fn new(allocation: Allocation, image: &SwashImage) -> Self {
        let rect = Rect::new(
            PPixel::new(image.placement.left as f32),
            PPixel::new(image.placement.top as f32),
            PPixel::new(image.placement.width as f32),
            PPixel::new(image.placement.height as f32),
        );

        let uv = Rect::new(
            allocation.rectangle.min.x as f32 / ATLAS_WIDTH as f32,
            allocation.rectangle.min.y as f32 / ATLAS_HEIGHT as f32,
            image.placement.width as f32 / ATLAS_WIDTH as f32,
            image.placement.height as f32 / ATLAS_HEIGHT as f32,
        );

        Self { rect, uv }
    }
}

pub struct PendingUpload {
    pub image: SwashImage,
    pub allocation: Allocation,
    pub atlas: usize,
}
