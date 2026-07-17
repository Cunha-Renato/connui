use bitflags::bitflags;

use crate::prelude::*;
use crate::{font::Font, image, types::Color};

pub trait Renderer: Sized {
    type ImageHandle: RendererImageHandle;

    fn record<I: IntoIterator<Item = RenderCommand<Self>>>(&mut self, commands: I);
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

pub enum RenderCommand<R: Renderer> {
    DrawRect {
        rect: Rect,
        uv: Option<Rect>,
        color: Color,
    },
    PushImage {
        id: Id,
        handle: R::ImageHandle,
    },
    PopImage,
    PushFont(Font<R>),
    PopFont,
}
