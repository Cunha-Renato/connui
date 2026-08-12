use connui::{
    image::Handle,
    impl_default_widget_layout,
    layout::WidgetDesc,
    prelude::*,
    renderer::{Renderer, RendererImageHandle},
    state::StateContext,
};

pub struct Image<T, R: Renderer> {
    gpu_handle: Option<R::ImageHandle>,
    load_handle: Handle<R>,
    size: Size<SizeOp>,
    _p: std::marker::PhantomData<T>,
}
impl<T, R: Renderer> Image<T, R> {
    #[inline]
    pub fn new(load_handle: Handle<R>) -> Self {
        Self {
            gpu_handle: None,
            load_handle,
            size: Size::default(),
            _p: Default::default(),
        }
    }
}
impl<T: 'static, R: Renderer + 'static> From<Image<T, R>> for Element<T, R> {
    #[inline]
    fn from(value: Image<T, R>) -> Self {
        Self::new(value)
    }
}

impl<T: 'static, R: Renderer + 'static> WidgetDiff for Image<T, R> {
    fn diff_eq(&self, other: Differ) -> bool {
        other.diff_eq(self, |a, b| {
            // We don't care about gpu texture because the diff is only done for the layout not rendering.
            a.load_handle.id() == b.load_handle.id() && a.size == b.size
        })
    }
}
impl<T, R: Renderer> WidgetDesc for Image<T, R> {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.size
    }
}
impl<T: 'static, R: Renderer + 'static> Widget<T, R> for Image<T, R> {
    fn init(&mut self, ctx: &mut StateContext<R>) {
        if let Some(gpu_handle) = ctx.renderer_mut().load_image(self.load_handle.clone()) {
            self.size.width = SizeOp::Absolute(LPixel::new(gpu_handle.width() as u16));
            self.size.height = SizeOp::Absolute(LPixel::new(gpu_handle.height() as u16));

            self.gpu_handle = Some(gpu_handle);
        }
    }

    fn render(&mut self, rect: &PRect, _: &Rect<u32, u32>, renderer: &mut R, _: &[Element<T, R>]) {
        if self.gpu_handle.is_some() {
            renderer.push_image(self.load_handle.id());
            renderer.draw_quad(rect, Color::from(0xffffffff), None);
            renderer.pop_image();
        }
    }

    fn get_children(&self) -> &[Element<T, R>] {
        &[]
    }

    fn get_children_mut(&mut self) -> &mut [Element<T, R>] {
        &mut []
    }
}

impl_default_widget_layout!({T: 'static, R: Renderer + 'static} Image {T, R});
