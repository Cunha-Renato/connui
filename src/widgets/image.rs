use crate::prelude::*;
use crate::renderer::{RenderCommand, RendererImageHandle};
use crate::{image, renderer::Renderer, widget::Widget};

pub struct Image<R: Renderer> {
    gpu_handle: Option<R::ImageHandle>,
    handle: image::Handle<R>,
    size: Size<SizeOp>,
}
impl<T: 'static, R: Renderer + 'static> Widget<T, R> for Image<R> {
    fn init(&mut self, ctx: &mut crate::state::StateContext<R>) {
        if let Some(gpu_handle) = ctx.load_image(self.handle.clone()) {
            self.size.width = SizeOp::Absolute(gpu_handle.width() as u16);
            self.size.height = SizeOp::Absolute(gpu_handle.height() as u16);

            self.gpu_handle = Some(gpu_handle);
        }
    }

    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.size
    }

    fn render(&self, position: Point, size: Size) -> Vec<RenderCommand<R>> {
        if let Some(gpu_handle) = self.gpu_handle.clone() {
            vec![
                RenderCommand::PushImage(gpu_handle),
                RenderCommand::DrawRect {
                    x: position.x,
                    y: position.y,
                    width: size.width,
                    height: size.height,
                    color: Color::from(0xffffffff),
                },
                RenderCommand::PopImage,
            ]
        } else {
            vec![]
        }
    }
}
impl<T: 'static, R: Renderer + 'static> From<Image<R>> for Element<T, R> {
    #[inline]
    fn from(value: Image<R>) -> Self {
        Self::new(value)
    }
}
impl<R: Renderer> Image<R> {
    #[inline]
    pub fn new(handle: image::Handle<R>) -> Self {
        Self {
            gpu_handle: None,
            handle,
            size: Size::default(),
        }
    }
}
