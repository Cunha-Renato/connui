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
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    },
    PushImage(R::ImageHandle),
    PopImage,
    PushFont(FontRef),
    PopFont,
}
