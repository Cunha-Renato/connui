use std::collections::HashMap;

use cosmic_text::SwashImage;

use crate::{image, renderer::Renderer, types::*};

use super::{Buffer, FontInner, atlas::*};

pub struct FontRenderer<'a, R: Renderer> {
    glyphs: HashMap<usize, Vec<GlyphQuad>>,
    font: &'a mut FontInner,
    renderer: &'a mut R,
}
impl<'a, R: Renderer> FontRenderer<'a, R> {
    #[inline]
    pub fn new(font: &'a mut FontInner, renderer: &'a mut R) -> Self {
        Self {
            glyphs: HashMap::default(),
            font,
            renderer,
        }
    }

    /// From `cosmic_text::Buffer::render`.
    pub fn render(&mut self, position: PPosition, buffer: &mut Buffer) {
        use cosmic_text::Renderer;
        let scale_factor = self.renderer.scale_factor();
        let color = cosmic_text::Color(0xffffffff);

        buffer.shape(&mut self.font.system);
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical_glyph = glyph.physical(
                    (
                        position.x.inner(),
                        position.y.inner() + run.line_y * scale_factor,
                    ),
                    scale_factor,
                );
                let glyph_color = glyph.color_opt.map_or(color, |some| some);

                self.glyph(physical_glyph, glyph_color);
            }

            cosmic_text::render_decoration(self, &run, color);
        }
    }

    pub fn finish(self) {
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
                self.renderer
                    .draw_quad(&glyph.cache.rect, glyph.color, Some(&glyph.cache.uv));
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
            .draw_quad(&rect, Color::from_hex_rgba(color.0), None);
    }

    fn glyph(&mut self, physical_glyph: cosmic_text::PhysicalGlyph, color: cosmic_text::Color) {
        let key = physical_glyph.cache_key;
        let Some((idx, cache)) = self.font.atlas_manager.get_or_maybe_insert(key, || {
            self.font
                .swash_cache
                .get_image_uncached(&mut self.font.system, key)
        }) else {
            return;
        };

        let mut cache = cache.clone();
        cache.rect.position.x += PPixel::new(physical_glyph.x as f32);
        cache.rect.position.y = PPixel::new(physical_glyph.y as f32) - cache.rect.position.y;

        self.glyphs.entry(*idx).or_default().push(GlyphQuad {
            cache,
            color: Color::from_hex_rgba(color.0),
        });
    }
}

struct GlyphQuad {
    cache: CachedGlyph,
    color: Color,
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
