use crate::{
    event::Event,
    renderer::RenderCommand,
    state::StateContext,
    types::{Layout, Point, Position, Response, Sides, Size, SizeOp},
};

pub trait Widget<T>: 'static {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        Size::default()
    }

    #[inline]
    fn get_position(&self) -> Position {
        Position::default()
    }

    #[inline]
    fn get_padding(&self) -> Sides<u16> {
        Sides::default()
    }

    #[inline]
    fn get_margin(&self) -> Sides<u16> {
        Sides::default()
    }

    #[inline]
    fn get_layout(&self) -> Layout {
        Layout::default()
    }

    /// This gets called only once a frame.
    /// There should be no problem just std::mem::take the children.
    fn get_children(&mut self) -> Vec<Element<T>> {
        vec![]
    }

    fn render(&self, position: Point, size: Size) -> Vec<RenderCommand>;

    #[inline]
    #[allow(unused_variables)]
    fn init(&mut self, ctx: &mut StateContext) {}

    #[inline]
    #[allow(unused_variables)]
    fn update(&mut self, ctx: &mut StateContext, position: Point, size: Size) {}

    #[inline]
    #[allow(unused_variables)]
    fn on_event(&mut self, event: Event) -> Response<T> {
        Default::default()
    }
}

pub struct Element<T>(Box<dyn Widget<T>>);
impl<T> Element<T> {
    #[inline]
    pub fn new(widget: impl Widget<T>) -> Self {
        Self(Box::new(widget))
    }
}
impl<T> From<Box<dyn Widget<T>>> for Element<T> {
    #[inline]
    fn from(value: Box<dyn Widget<T>>) -> Self {
        Self(value)
    }
}
impl<T> std::ops::Deref for Element<T> {
    type Target = Box<dyn Widget<T>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> std::ops::DerefMut for Element<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
