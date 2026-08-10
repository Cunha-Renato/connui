use crate::prelude::*;
use crate::{image, types::Color};

pub trait Renderer: Sized {
    type ImageHandle: RendererImageHandle;

    fn scale_factor(&self) -> f32;
    fn set_scale_factor(&mut self, scale_factor: f32);

    // Render Commands.
    fn draw_quad(&mut self, rect: &PRect, color: Color, uv: Option<&Rect<f32, f32>>);

    fn push_scissor(&mut self, rect: &Rect<u32, u32>);
    fn pop_scissor(&mut self);

    fn push_image(&mut self, id: Id);
    fn pop_image(&mut self);

    fn load_image(&mut self, handle: image::Handle<Self>) -> Option<Self::ImageHandle>;
    fn write_image<'a>(
        &mut self,
        id: &Id,
        write_op: image::WriteOp<'a>,
        fallback: Option<image::Handle<Self>>,
    );
}

pub trait RendererImageHandle: Clone + Send + Sync {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}
