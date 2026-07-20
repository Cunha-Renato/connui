use bitflags::bitflags;

use crate::prelude::*;
use crate::{font::Font, image, types::Color};

pub trait Renderer: Sized {
    type ImageHandle: RendererImageHandle;

    // Render Commands.
    fn draw_quad(&mut self, rect: Rect, color: Color, uv: Option<Rect>);
    fn draw_char(&mut self, rect: Rect, color: Color, char: char);

    fn push_image(&mut self, handle: image::Handle<Self>);
    fn pop_image(&mut self);

    fn push_font(&mut self, font: Font<Self>);
    fn pop_font(&mut self);

    fn begin_text(&mut self);
    fn end_text(&mut self);

    fn load_image(&mut self, handle: image::Handle<Self>) -> Option<Self::ImageHandle>;
    fn supported_font_render_method(&self) -> FontRenderMethod;
}

pub trait RendererImageHandle: Clone + Send + Sync {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

bitflags! {
    pub struct FontRenderMethod: u8 {
        const BITMAP = 0b1;
        const SDF = 0b10;
        const MSDF = 0b100;
    }
}
