use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{image, renderer::Renderer, types::*};
use cosmic_text::{
    Attrs, Buffer, CacheKey, FontSystem, Metrics, PhysicalGlyph, Shaping, SwashCache, SwashImage,
};
use etagere::{Allocation, AtlasAllocator, euclid::Size2D};

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
    pub fn layout(&self, text_size: u16) -> Layout {
        Layout::new(self.clone(), text_size)
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

pub struct Layout {
    buffer: Buffer,
    prepared: HashMap<usize, Vec<CachedGlyph>>,
    font: Font,
}
impl Layout {
    #[inline]
    fn new(font: Font, text_size: u16) -> Self {
        let mut buffer = font.lock(|inner| {
            let text_size = text_size as f32;

            Buffer::new(&mut inner.system, Metrics::new(text_size, text_size * 1.25))
        });
        buffer.set_wrap(cosmic_text::Wrap::Word);

        Self {
            buffer,
            prepared: HashMap::new(),
            font,
        }
    }

    /// Changes the metrics used by this layout.
    #[inline]
    pub fn set_metrics(&mut self, text_size: u16, line_height: u16) {
        self.buffer
            .set_metrics(Metrics::new(text_size as f32, line_height as f32));
    }

    /// Returns the metrics currently used by this layout.
    #[inline]
    pub fn metrics(&self) -> (u16, u16) {
        let metrics = self.buffer.metrics();

        (metrics.font_size as u16, metrics.line_height as u16)
    }

    /// Shapes and lays out the provided text.
    pub fn set_text(&mut self, text: &str) {
        self.buffer
            .set_text(text, &Attrs::new(), Shaping::Advanced, None);
    }

    /// Shapes text constrained to the given width.
    pub fn set_text_with_width(&mut self, text: &str, width: f32) {
        self.buffer.set_size(Some(width), self.buffer.size().1);
        self.buffer
            .set_text(text, &Attrs::new(), Shaping::Advanced, None);
    }

    /// Returns the size occupied by the current layout.
    pub fn size(&self) -> Size {
        let mut size: Size = Size::default();

        for run in self.buffer.layout_runs() {
            size.width = size.width.max(run.line_w);
            size.height = size.height.max(run.line_top + run.line_height);
        }

        size
    }

    #[inline]
    pub fn set_size(&mut self, width: Option<u16>, height: Option<u16>) {
        self.buffer
            .set_size(width.map(|w| w as f32), height.map(|h| h as f32));
    }

    /// Returns true if no glyphs are present.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.layout_runs().next().is_none()
    }

    /// Removes all text from the layout.
    pub fn clear(&mut self) {
        self.buffer.set_size(None, None);
        self.buffer
            .set_text("", &Attrs::new(), Shaping::Advanced, None);
    }

    pub fn shape(&mut self) {
        self.font.lock(|font| {
            self.buffer.shape_until_scroll(&mut font.system, true);
        });
    }

    /// Draws the current layout.
    pub fn render<R: Renderer>(&mut self, renderer: &mut R, position: Point<f32>, color: Color) {
        self.prepare(renderer.scale_factor());

        self.font.lock(|font| {
            for upload in font.atlas_manager.take_uploads() {
                let atlas = font.atlas_manager.atlas(upload.atlas);

                let img_rect = Rect::new(
                    upload.allocation.rectangle.min.x as u32,
                    upload.allocation.rectangle.min.y as u32,
                    upload.image.placement.width,
                    upload.image.placement.height,
                );
                renderer.write_image(
                    &atlas.id,
                    crate::image::WriteOp {
                        rect: img_rect,
                        bytes: &into_rgba(upload.image),
                    },
                    Some(image::Handle::once(atlas.id, || {
                        let bytes = vec![0u8; ATLAS_WIDTH * ATLAS_HEIGHT * 4];

                        image::HandleKind::Custom {
                            width: ATLAS_WIDTH as u32,
                            height: ATLAS_HEIGHT as u32,
                            bpp: 4,
                            bytes: bytes.into(),
                        }
                    })),
                );
            }

            for (atlas, glyphs) in &self.prepared {
                renderer.push_image(font.atlas_manager.atlas(*atlas).id);
                for glyph in glyphs {
                    let mut rect = glyph.rect;
                    rect.position.x += position.x;
                    rect.position.y += position.y;

                    renderer.draw_quad(rect, color, Some(glyph.uv));
                }
                renderer.pop_image();
            }
        });
    }

    fn prepare(&mut self, scale_factor: f32) {
        self.prepared.clear();
        self.font.lock(|font| {
            self.buffer.shape_until_scroll(&mut font.system, true);

            for run in self.buffer.layout_runs() {
                for glyph in run.glyphs {
                    // TODO: Scale factor.
                    let physical = glyph.physical((0.0, run.line_y), scale_factor);

                    let key = physical.cache_key;
                    let (idx, mut cache) = match font.atlas_manager.get(&key) {
                        Some(cache) => cache,
                        None => {
                            let Some(image) = font
                                .swash_cache
                                .get_image_uncached(&mut font.system, physical.cache_key)
                            else {
                                continue;
                            };

                            if image.placement.width == 0 || image.placement.height == 0 {
                                continue;
                            }

                            font.atlas_manager.insert(key, image)
                        }
                    };

                    cache.rect.position.x += physical.x as f32;
                    cache.rect.position.y = physical.y as f32 - cache.rect.position.y;

                    self.prepared
                        .entry(idx)
                        .or_insert_with(|| vec![])
                        .push(cache);
                }
            }
        });
    }
}

// ATLAS.

const ATLAS_WIDTH: usize = 2048;
const ATLAS_HEIGHT: usize = 2048;

struct AtlasManager {
    glyphs: HashMap<CacheKey, (usize, CachedGlyph)>,
    atlases: Vec<Atlas>,
    uploads: Vec<PendingUpload>,
}
impl AtlasManager {
    #[inline]
    fn new() -> Self {
        Self {
            atlases: vec![Atlas::new(Id::new("connui_font::atlas::0"))],
            glyphs: HashMap::default(),
            uploads: Vec::new(),
        }
    }

    fn get(&self, key: &CacheKey) -> Option<(usize, CachedGlyph)> {
        self.glyphs.get(key).copied()
    }

    fn insert(&mut self, key: CacheKey, image: SwashImage) -> (usize, CachedGlyph) {
        let size = Size2D::new(image.placement.width as i32, image.placement.height as i32);

        loop {
            let atlas = self.atlases.last_mut().unwrap();

            if let Some(allocation) = atlas.allocator.allocate(size) {
                let cached = CachedGlyph::new(allocation, &image);
                let atlas_idx = self.atlases.len() - 1;

                self.glyphs.insert(key, (atlas_idx, cached));

                self.uploads.push(PendingUpload {
                    atlas: atlas_idx,
                    allocation,
                    image,
                });

                return (atlas_idx, cached);
            }

            let id = Id::new(format!("connui_font::atlas::{}", self.atlases.len()));
            self.atlases.push(Atlas::new(id));
        }
    }

    #[inline]
    fn atlas(&self, index: usize) -> &Atlas {
        &self.atlases[index]
    }

    #[inline]
    fn take_uploads(&mut self) -> Vec<PendingUpload> {
        std::mem::take(&mut self.uploads)
    }
}
impl Default for AtlasManager {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

struct Atlas {
    allocator: AtlasAllocator,
    id: Id,
}
impl Atlas {
    fn new(id: Id) -> Self {
        Self {
            id,
            allocator: AtlasAllocator::new(Size2D::new(ATLAS_WIDTH as i32, ATLAS_HEIGHT as i32)),
        }
    }
}

#[derive(Clone, Copy)]
struct CachedGlyph {
    rect: Rect,
    uv: Rect,
}
impl CachedGlyph {
    fn new(allocation: Allocation, image: &SwashImage) -> Self {
        let rect = Rect::new(
            image.placement.left as f32,
            image.placement.top as f32,
            image.placement.width as f32,
            image.placement.height as f32,
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

struct PendingUpload {
    image: SwashImage,
    allocation: Allocation,
    atlas: usize,
}

fn into_rgba(image: SwashImage) -> Vec<u8> {
    use cosmic_text::SwashContent;

    match image.content {
        SwashContent::Color => image.data,
        SwashContent::Mask => {
            let mut rgba = Vec::with_capacity(
                image.placement.width as usize * image.placement.height as usize * 4,
            );

            for alpha in image.data {
                rgba.extend_from_slice(&[0xff, 0xff, 0xff, alpha]);
            }

            rgba
        }
        SwashContent::SubpixelMask => {
            let mut rgba = Vec::with_capacity(
                image.placement.width as usize * image.placement.height as usize * 4,
            );

            for rgb in image.data.chunks_exact(3) {
                rgba.extend_from_slice(&[0xff, 0xff, 0xff, rgb[0]]);
            }

            rgba
        }
    }
}
