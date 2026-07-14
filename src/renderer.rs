use crate::prelude::*;
use crate::{font::FontRef, image, types::Color};

pub trait RendererImageHandle: Clone + Send + Sync {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

pub trait Renderer: Sized {
    type ImageHandle: RendererImageHandle;

    fn load_image(&mut self, handle: image::Handle<Self>) -> Option<Self::ImageHandle>;
}

pub enum RenderCommand<R: Renderer> {
    DrawRect {
        rect: Rect,
        uv: Option<Rect>,
        color: Color,
    },
    PushImage(R::ImageHandle),
    PopImage,
    PushFont(FontRef),
    PopFont,
}
