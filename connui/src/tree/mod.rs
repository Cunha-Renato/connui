pub mod layout;
pub mod visual;
pub mod widget;

use std::any::Any;

use crate::{
    event::{Event, EventKind, InputContext},
    renderer::Renderer,
    types::Response,
};

pub use layout::*;
pub use visual::*;
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

    fn input_event(&mut self, event: Event, layout_element: &LayoutElement) -> Response<T>;

    fn input_consumed(&mut self, kind: EventKind);

    fn input_capture(&self) -> EventKind;

    fn render(
        &self,
        children: &[VisualElement<T, R>],
        render_element: RenderElement,
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
