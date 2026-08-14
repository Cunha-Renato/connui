pub mod diff;

use crate::{
    event::Event,
    layout::{LayoutElement, WidgetLayout},
    prelude::*,
    renderer::Renderer,
    state::StateContext,
};

pub use diff::{AsAny, Differ, WidgetDiff};

pub trait Widget<T, R: Renderer>: WidgetLayout + WidgetDiff {
    fn get_children(&self) -> &[Element<T, R>];
    fn get_children_mut(&mut self) -> &mut [Element<T, R>];

    #[inline]
    #[allow(unused_variables)]
    fn render(
        &mut self,
        render_element: RenderElement,
        layout_element: &LayoutElement,
        renderer: &mut R,
    ) {
        if !self.get_children().is_empty() {
            renderer.push_scissor(&render_element.scissor);
            for (child, child_layout) in self
                .get_children_mut()
                .iter_mut()
                .zip(&layout_element.children)
            {
                child.render(child_layout, renderer);
            }
            renderer.pop_scissor();
        }
    }

    #[inline]
    #[allow(unused_variables)]
    fn init(&mut self, ctx: &mut StateContext<R>) {}

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
    pub(crate) fn layout(&mut self) -> LayoutElement {
        let mut layout_element = LayoutElement::new(self);

        self.widget
            .measure(&mut layout_element, &Default::default());
        self.widget.resolve(&mut layout_element);

        layout_element
    }

    #[inline]
    pub fn render(&mut self, layout_element: &LayoutElement, renderer: &mut R) {
        let render_element = RenderElement::new(layout_element, renderer);

        if !render_element.is_visible() {
            return;
        }

        self.widget.render(render_element, layout_element, renderer);
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

pub struct RenderElement {
    pub rect: PRect,
    pub scissor: Rect<u32, u32>,
    visible: bool,
}
impl RenderElement {
    pub fn new<R: Renderer>(layout_element: &LayoutElement, renderer: &R) -> Self {
        let scale_factor = renderer.scale_factor();
        let rect = layout_element.rect.map(|lp| lp.to_physical(scale_factor));
        let clip = layout_element.clip.map(|lp| lp.to_physical(scale_factor));

        // Clipped away.
        let visible = rect.intersects(&clip);

        let min_x = clip.x().inner().floor();
        let min_y = clip.y().inner().floor();
        let max_x = (clip.x() + clip.width()).inner().ceil();
        let max_y = (clip.y() + clip.height()).inner().ceil();

        let scissor = Rect::new(
            min_x as u32,
            min_y as u32,
            (max_x - min_x) as u32,
            (max_y - min_y) as u32,
        );

        Self {
            rect,
            scissor,
            visible,
        }
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}
