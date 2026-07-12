use crate::{font::FontRef, image, types::Color};

pub trait Renderer: Sized {
    type ImageHandle: Send + Sync;

    fn draw_commands<I: IntoIterator<Item = RenderCommand<Self>>>(&mut self, commands: I);
}

pub enum RenderCommand<R: Renderer> {
    DrawRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    },
    SetImage {
        handle: image::Handle<R>,
    },
    SetFont(FontRef),
}
