pub mod diff;

use crate::{
    event::Event, layout::WidgetLayout, prelude::*, renderer::Renderer, state::StateContext,
};

pub use diff::{AsAny, Differ, WidgetDiff};

pub trait Widget<T, R: Renderer>: WidgetLayout + WidgetDiff {
    fn get_children(&self) -> &[Element<T, R>];
    fn get_children_mut(&mut self) -> &mut [Element<T, R>];

    #[inline]
    #[allow(unused_variables)]
    fn render(
        &mut self,
        rect: &PRect,
        scissor: &Rect<u32, u32>,
        renderer: &mut R,
        children: &[Element<T, R>],
    ) {
        renderer.push_scissor(scissor);
        for child in children {
            // child.render(renderer);
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
        rect: LRect<i32, u16>,
        clip: LRect<i32, u16>,
        children: &[Element<T, R>],
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

pub struct Element<T, R: Renderer> {
    widget: Box<dyn Widget<T, R>>,
    tid: std::any::TypeId,
}
impl<T, R: Renderer> Element<T, R> {
    #[inline]
    pub fn new<W: Widget<T, R> + 'static>(widget: W) -> Self {
        Self {
            widget: Box::new(widget),
            tid: std::any::TypeId::of::<W>(),
        }
    }

    pub(crate) fn init(&mut self, ctx: &mut StateContext<R>) {
        self.widget.init(ctx);

        for child in self.widget.get_children_mut() {
            child.init(ctx);
        }
    }

    #[inline]
    pub(crate) fn diff(&self, other: &diff::DiffElement) -> bool {
        self.differ() == other.differ()
    }

    #[inline]
    pub(crate) fn layout(&mut self) -> crate::layout::LayoutElement {
        let mut layout_element = crate::layout::LayoutElement::new(self);

        self.widget
            .measure(&mut layout_element, &Default::default());
        self.widget.resolve_children_size(&mut layout_element);
        self.widget.resolve_children_position(&mut layout_element);

        layout_element
    }

    #[inline]
    fn differ(&self) -> Differ<'_> {
        Differ::new(self.widget.as_ref(), &self.tid)
    }
}
impl<T, R: Renderer> std::ops::Deref for Element<T, R> {
    type Target = dyn Widget<T, R>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.widget.as_ref()
    }
}
impl<T, R: Renderer> std::ops::DerefMut for Element<T, R> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.widget.as_mut()
    }
}
impl<T, R: Renderer> PartialEq for Element<T, R> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.tid == other.tid && self.widget.diff_eq(self.differ())
    }
}
