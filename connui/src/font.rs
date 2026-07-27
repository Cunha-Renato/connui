use crate::{image, prelude::*, renderer::Renderer};
use std::sync::Arc;

pub struct Font<R: Renderer>(Arc<dyn FontSpecs<R>>);
impl<R: Renderer> Font<R> {
    #[inline]
    pub fn new<F: FontSpecs<R> + 'static>(font: F) -> Self {
        Self(Arc::new(font))
    }
}
impl<R: Renderer> Clone for Font<R> {
    #[inline]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
impl<R: Renderer> std::ops::Deref for Font<R> {
    type Target = dyn FontSpecs<R>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub trait FontSpecs<R: Renderer> {
    // Metrics
    fn data(&self, text_size: u16, text: &str) -> Vec<GlyphData>;
    fn new_line(&self, text_size: u16) -> Point<i16>;
    fn ascender(&self, text_size: u16) -> Point<i16>;
    fn descender(&self, text_size: u16) -> Point<i16>;

    // Rendering.
    fn atlas(&self, text_size: u16) -> image::Handle<R>;
}

pub struct GlyphData {
    pub uv: Rect,
    pub size: Size<u16>,
    pub advance: Point<i16>,
    pub bearing: Point<i16>,
}
