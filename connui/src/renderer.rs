use crate::prelude::*;
use crate::{font::Font, image, types::Color};

pub trait Renderer: Sized {
    type ImageHandle: RendererImageHandle;

    // Render Commands.
    fn draw_quad(&mut self, rect: Rect, color: Color, uv: Option<Rect>);
    fn draw_char(&mut self, rect: Rect, color: Color, char: char);

    fn push_scissor(&mut self, rect: Rect);
    fn pop_scissor(&mut self);

    fn push_image(&mut self, id: Id);
    fn pop_image(&mut self);

    fn begin_text(&mut self, font: Font<Self>, size: u16);
    fn end_text(&mut self);

    fn load_image(&mut self, handle: image::Handle<Self>) -> Option<Self::ImageHandle>;
}

pub trait RendererImageHandle: Clone + Send + Sync {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}
