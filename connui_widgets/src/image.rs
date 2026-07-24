use connui::{
    image::Handle,
    prelude::*,
    renderer::{Renderer, RendererImageHandle},
    state::StateContext,
};

pub struct Image<R: Renderer> {
    gpu_handle: Option<R::ImageHandle>,
    load_handle: Handle<R>,
    size: Size<SizeOp>,
}
impl<T: 'static, R: Renderer + 'static> Widget<T, R> for Image<R> {
    fn init(&mut self, ctx: &mut StateContext<R>) {
        if let Some(gpu_handle) = ctx.renderer_mut().load_image(self.load_handle.clone()) {
            self.size.width = SizeOp::Absolute(gpu_handle.width() as u16);
            self.size.height = SizeOp::Absolute(gpu_handle.height() as u16);

            self.gpu_handle = Some(gpu_handle);
        }
    }

    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.size
    }

    fn render(&self, rect: Rect, _: Rect, renderer: &mut R, _: &[Node<T, R>]) {
        if self.gpu_handle.is_some() {
            renderer.push_image(self.load_handle.id());
            renderer.draw_quad(rect, Color::from(0xffffffff), None);
            renderer.pop_image();
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
    pub fn new(load_handle: Handle<R>) -> Self {
        Self {
            gpu_handle: None,
            load_handle,
            size: Size::default(),
        }
    }
}
