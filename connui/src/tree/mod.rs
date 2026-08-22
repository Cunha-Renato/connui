pub mod layout;
pub mod widget;

use std::any::Any;

use crate::{
    event::{Event, InputContext},
    renderer::Renderer,
    types::{PRect, Rect, Response},
};

pub use layout::*;
pub use widget::*;

#[repr(transparent)]
struct Differ<'a>(&'a dyn Any);
impl PartialEq<Key> for Differ<'_> {
    #[inline]
    fn eq(&self, key: &Key) -> bool {
        key == self.0.type_id()
    }
}

#[repr(transparent)]
pub struct Updater<'a>(&'a mut dyn Any);
impl<'a> Updater<'a> {
    #[inline]
    pub fn update<W, E: 'static, F>(self, widget: W, f: F)
    where
        F: FnOnce(W, &mut E),
    {
        if let Some(element) = self.0.downcast_mut::<E>() {
            f(widget, element)
        }
    }
}

pub trait ElementSpecs<T, R: Renderer>: Any {
    fn style(&self) -> &Style;

    #[allow(unused_variables)]
    fn layout(&mut self, layout_key: LayoutElementKey, layout_tree: &mut LayoutElementTree) {}

    fn event(
        &mut self,
        event: Event,
        input_context: &mut InputContext,
        layout_element: &LayoutElement,
    ) -> Response<T>;

    fn render(
        &self,
        render_element: RenderElement,
        children: &[Element<T, R>],
        layout_tree: &LayoutElementTree,
        renderer: &mut R,
    );
}

pub struct Element<T, R: Renderer> {
    pub(crate) children: Vec<Self>,
    element: Box<dyn ElementSpecs<T, R>>,
    layout: Option<LayoutElementKey>,
}
impl<T: 'static, R: Renderer + 'static> Element<T, R> {
    pub fn render(&self, layout_tree: &LayoutElementTree, renderer: &mut R) {
        let layout_key = self.layout.unwrap();
        let render_element = RenderElement::new(&layout_tree[layout_key], renderer);

        self.element
            .render(render_element, &self.children, layout_tree, renderer);
    }

    pub(crate) fn layout(&mut self, layout_tree: &mut LayoutElementTree) {
        self.element.layout(self.layout.unwrap(), layout_tree);

        for child in &mut self.children {
            child.layout(layout_tree);
        }
    }

    pub(crate) fn reconcile(
        &mut self,
        layout_tree: &mut LayoutElementTree,
        mut widget: Widget<T, R>,
    ) {
        if self.differ() == widget.key() {
            let children = widget.children();

            widget.update(self.updater());
            self.validate_style(layout_tree);
            self.reconcile_children(layout_tree, children);
        } else {
            *self = widget.mount();
            layout_tree.dirty();
        }
    }

    fn reconcile_children(
        &mut self,
        layout_tree: &mut LayoutElementTree,
        new_children: Vec<Widget<T, R>>,
    ) {
        if self.children.len() != new_children.len() {
            layout_tree.dirty();
        }

        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        }

        let mut new_children = new_children.into_iter();

        for (old, new) in self.children.iter_mut().zip(new_children.by_ref()) {
            old.reconcile(layout_tree, new);
        }

        self.children.extend(new_children.map(Widget::mount));
    }

    /// Marks tree as dirty if [`Style`] is different from [`LayoutElement`].
    fn validate_style(&mut self, layout_tree: &mut LayoutElementTree) {
        if let Some(layout_key) = self.layout {
            let inner_style = self.element.style().into();

            if layout_tree[layout_key].style == inner_style {
                layout_tree[layout_key].style = inner_style;
                layout_tree.dirty();
            }
        }
    }
}
impl<T, R: Renderer> Element<T, R> {
    #[inline]
    pub fn new<E, I>(children: I, element: E) -> Self
    where
        E: ElementSpecs<T, R> + 'static,
        I: IntoIterator<Item = Widget<T, R>>,
    {
        Self {
            children: children.into_iter().map(Widget::mount).collect(),
            element: Box::new(element),
            layout: None,
        }
    }

    #[inline]
    pub fn children(&self) -> &[Self] {
        &self.children
    }

    #[inline]
    fn differ<'a>(&'a self) -> Differ<'a> {
        Differ(self.element.as_ref())
    }

    #[inline]
    fn updater<'a>(&'a mut self) -> Updater<'a> {
        Updater(self.element.as_mut())
    }
}

#[derive(Debug)]
pub struct RenderElement {
    pub rect: PRect,
    pub scissor: Rect<u32, u32>,
}
impl RenderElement {
    /// QOL function to check if the scissor is visible.
    #[inline]
    pub const fn can_render_children(&self) -> bool {
        self.scissor.x() < self.scissor.width() && self.scissor.y() < self.scissor.width()
    }

    fn new<R: Renderer>(layout_element: &LayoutElement, renderer: &mut R) -> Self {
        let scale_factor = renderer.scale_factor();

        let rect = layout_element.rect.map(|r| r.to_physical(scale_factor));
        let clip_inner = layout_element
            .clip
            .map(|c| c.to_physical(scale_factor).inner());

        let x0 = clip_inner.x();
        let y0 = clip_inner.y();
        let x1 = clip_inner.x() + clip_inner.width();
        let y1 = clip_inner.y() + clip_inner.height();

        let scissor_x = x0.floor().max(0.0) as u32;
        let scissor_y = y0.floor().max(0.0) as u32;
        let scissor_w = (x1 - x0).ceil().max(0.0) as u32;
        let scissor_h = (y1 - y0).ceil().max(0.0) as u32;

        let scissor = Rect::new(scissor_x, scissor_y, scissor_w, scissor_h);

        Self { rect, scissor }
    }
}
