use crate::{image, renderer::Renderer, types::*};
use cosmic_text::{CacheKey, FontSystem, Metrics, SwashCache, SwashImage};
use etagere::{Allocation, AtlasAllocator, euclid::Size2D};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
/// All positions and sizes returned by this type (other than internal
/// rendering state) are in **logical** pixels — i.e. unaffected by display
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

    /// Draws the current layout at `position` (logical pixels, top-left).
    pub fn render<R: Renderer>(&mut self, renderer: &mut R, position: Point) {
        self.font.lock(|inner| {
            let mut font_renderer = FontRenderer::new(inner, renderer);
            font_renderer.render(&mut self.buffer);
            font_renderer.finish(position);
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

#[derive(Debug)]
pub struct Buffer(cosmic_text::Buffer);
impl Buffer {
    #[inline]
    fn new(font_system: &mut FontSystem) -> Self {
        Self(cosmic_text::Buffer::new(
            font_system,
            Metrics::new(16.0, 16.0 * 1.5),
        ))
    }

    #[inline]
    pub fn set_font_size(&mut self, size: LogicalPixel) {
        let old = self.0.metrics();
        let old_mult = old.line_height / old.font_size;
        let new_size = size.inner() as f32;
        self.0
            .set_metrics(Metrics::new(new_size, new_size * old_mult));
    }

    #[inline]
    pub fn set_line_height_mult(&mut self, mult: f32) {
        let old = self.0.metrics();
        let new_line_height = old.font_size * mult;
        self.0
            .set_metrics(Metrics::new(old.font_size, new_line_height));
    }

    #[inline]
    pub fn set_metrics(&mut self, font_size: LogicalPixel, line_height_mult: f32) {
        let font_size = font_size.inner() as f32;
        self.0
            .set_metrics(Metrics::new(font_size, font_size * line_height_mult));
    }

    #[inline]
    pub fn set_bounding_box(&mut self, size: Size<Option<LogicalPixel>>) {
        self.0.set_size(
            size.width.map(|w| w.inner() as f32),
            size.height.map(|h| h.inner() as f32),
        );
    }

    #[inline]
    pub fn bounding_box(&self) -> Size<Option<LogicalPixel>> {
        let size = self.0.size();

        Size::new(
            size.0.map(|w| LogicalPixel::new(w as u16)),
            size.1.map(|h| LogicalPixel::new(h as u16)),
        )
    }

    #[inline]
    pub fn shaped_size(&mut self) -> Size<LogicalPixel> {
        let mut width = 0;
        let mut height = 0;

        for run in self.0.layout_runs() {
            width = width.max(run.line_w as u16);
            height = height.max((run.line_top + run.line_height) as u16);
        }

        Size::new(width.into(), height.into())
    }

    #[inline]
    pub fn text(&mut self, specs: &TextSpecs) {
        self.0.set_text(
            &specs.text,
            &specs.attrs,
            cosmic_text::Shaping::Advanced,
            Some(specs.alignment),
        );
    }

    #[inline]
    fn shape(&mut self, font_system: &mut FontSystem) {
        if self.0.redraw() {
            self.0.shape_until_scroll(font_system, true);
        }
    }
}

#[derive(Clone)]
pub struct TextSpecs<'a> {
    attrs: cosmic_text::Attrs<'a>,
    alignment: cosmic_text::Align,
    text: std::borrow::Cow<'a, str>,
}
impl<'a> TextSpecs<'a> {
    #[inline]
    pub fn new<S: Into<std::borrow::Cow<'a, str>>>(text: S) -> Self {
        let attrs = cosmic_text::Attrs::new()
            .underline(cosmic_text::UnderlineStyle::Single)
            .underline_color(cosmic_text::Color(0x00ffffff))
            .overline()
            .strikethrough()
            .weight(cosmic_text::Weight(700));

        Self {
            attrs,
            alignment: cosmic_text::Align::Left,
            text: text.into(),
        }
    }
}

// ATLAS.

const ATLAS_WIDTH: usize = 1024;
const ATLAS_HEIGHT: usize = 1024;

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
    uv: Rect<f32, f32>,
}
impl CachedGlyph {
    fn new(allocation: Allocation, image: &SwashImage) -> Self {
        let rect = Rect::new(
            PhysicalPixel::new(image.placement.left as f32),
            PhysicalPixel::new(image.placement.top as f32),
            PhysicalPixel::new(image.placement.width as f32),
            PhysicalPixel::new(image.placement.height as f32),
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

// RENDERER

struct FontRenderer<'a, R: Renderer> {
    glyphs: HashMap<usize, Vec<GlyphQuad>>,
    font: &'a mut FontInner,
    renderer: &'a mut R,
}
impl<'a, R: Renderer> FontRenderer<'a, R> {
    #[inline]
    fn new(font: &'a mut FontInner, renderer: &'a mut R) -> Self {
        Self {
            glyphs: HashMap::default(),
            font,
            renderer,
        }
    }

    /// From `cosmic_text::Buffer::render`.
    fn render(&mut self, buffer: &mut Buffer) {
        use cosmic_text::Renderer;
        let scale_factor = self.renderer.scale_factor();
        let color = cosmic_text::Color(0xffffffff);

        buffer.0.shape_until_scroll(&mut self.font.system, false);
        for run in buffer.0.layout_runs() {
            for glyph in run.glyphs {
                let physical_glyph = glyph.physical((0.0, run.line_y * scale_factor), scale_factor);
                let glyph_color = glyph.color_opt.map_or(color, |some| some);

                self.glyph(physical_glyph, glyph_color);
            }

            cosmic_text::render_decoration(self, &run, color);
        }
    }

    fn finish(self, position: Point) {
        // GLYPHS
        for upload in self.font.atlas_manager.take_uploads() {
            let atlas = self.font.atlas_manager.atlas(upload.atlas);

            let img_rect = Rect::new(
                upload.allocation.rectangle.min.x as u32,
                upload.allocation.rectangle.min.y as u32,
                upload.image.placement.width,
                upload.image.placement.height,
            );
            self.renderer.write_image(
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

        for (atlas, glyphs) in &self.glyphs {
            self.renderer
                .push_image(self.font.atlas_manager.atlas(*atlas).id);
            for glyph in glyphs {
                let mut g_position = glyph.cache.rect.position;
                let g_size = glyph.cache.rect.size;

                g_position.x += position.x;
                g_position.y += position.y;

                self.renderer.draw_quad(
                    Rect::new_pos_size(g_position, g_size),
                    glyph.color,
                    Some(glyph.cache.uv),
                );
            }
            self.renderer.pop_image();
        }
    }
}
impl<'a, R: Renderer> cosmic_text::Renderer for FontRenderer<'a, R> {
    fn rectangle(&mut self, x: i32, y: i32, w: u32, h: u32, color: cosmic_text::Color) {
        let scale_factor = self.renderer.scale_factor();

        let rect = Rect::new(
            (x as f32 * scale_factor).into(),
            (y as f32 * scale_factor).into(),
            (w as f32 * scale_factor).into(),
            (h as f32 * scale_factor).into(),
        );
        self.renderer
            .draw_quad(rect, Color::from_hex(color.0), None);
    }

    fn glyph(&mut self, physical_glyph: cosmic_text::PhysicalGlyph, color: cosmic_text::Color) {
        let key = physical_glyph.cache_key;
        let Some((idx, mut cache)) = self.font.atlas_manager.get(&key).or_else(|| {
            match self
                .font
                .swash_cache
                .get_image_uncached(&mut self.font.system, key)
            {
                Some(img) if img.placement.width > 0 && img.placement.height > 0 => {
                    Some(self.font.atlas_manager.insert(key, img))
                }
                _ => None,
            }
        }) else {
            return;
        };

        cache.rect.position.x += PhysicalPixel::new(physical_glyph.x as f32);
        cache.rect.position.y = PhysicalPixel::new(physical_glyph.y as f32) - cache.rect.position.y;

        self.glyphs.entry(idx).or_default().push(GlyphQuad {
            cache,
            color: Color::from_hex(color.0),
        });
    }
}

struct GlyphQuad {
    cache: CachedGlyph,
    color: Color,
}
