use crate::{event::Event, prelude::*, renderer::Renderer, state::StateContext};

pub trait Widget<T, R: Renderer> {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        Size::default()
    }

    #[inline]
    fn get_position(&self) -> Position {
        Position::default()
    }

    #[inline]
    fn get_padding(&self) -> Sides<LogicalPixel> {
        Sides::default()
    }

    #[inline]
    fn get_margin(&self) -> Sides<LogicalPixel> {
        Sides::default()
    }

    #[inline]
    fn get_layout(&self) -> Layout {
        Layout::default()
    }

    /// This gets called only once a frame.
    /// There should be no problem just std::mem::take the children.
    fn get_children(&mut self) -> Vec<Element<T, R>> {
        vec![]
    }

    #[inline]
    #[allow(unused_variables)]
    fn render(
        &mut self,
        rect: Rect,
        scissor: Rect<u32, u32>,
        renderer: &mut R,
        children: &mut [Node<T, R>],
    ) {
        renderer.push_scissor(scissor);
        for child in children {
            child.render(renderer);
        }
        renderer.pop_scissor();
    }

    #[inline]
    #[allow(unused_variables)]
    fn init(&mut self, ctx: &mut StateContext<R>) {}

    #[inline]
    #[allow(unused_variables)]
    /// Returns [`true`] if layout is invalid, if so the layout engine will run again.
    fn update(
        &mut self,
        rect: Rect<LogicalPixel<i32>, LogicalPixel>,
        ctx: &mut StateContext<R>,
    ) -> bool {
        false
    }

    #[inline]
    #[allow(unused_variables)]
    fn on_event(&mut self, event: Event) -> Response<T> {
        Default::default()
    }
}

pub struct Element<T, R: Renderer>(Box<dyn Widget<T, R>>);
impl<T, R: Renderer> Element<T, R> {
    #[inline]
    pub fn new(widget: impl Widget<T, R> + 'static) -> Self {
        Self(Box::new(widget))
    }
}
impl<T, R: Renderer> From<Box<dyn Widget<T, R>>> for Element<T, R> {
    #[inline]
    fn from(value: Box<dyn Widget<T, R>>) -> Self {
        Self(value)
    }
}
impl<T, R: Renderer> std::ops::Deref for Element<T, R> {
    type Target = Box<dyn Widget<T, R>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T, R: Renderer> std::ops::DerefMut for Element<T, R> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
