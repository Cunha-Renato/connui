pub mod layout;
pub mod visual;
pub mod widget;

use std::any::Any;

use crate::{
    event::{EventKind, MouseEvent, WindowEvent},
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

pub struct Updater<'a> {
    element: &'a mut dyn Any,
}
impl<'a> Updater<'a> {
    #[inline]
    pub fn update<W, E: 'static, F>(&mut self, widget: W, f: F)
    where
        F: FnOnce(W, &mut E),
    {
        if let Some(element) = self.element.downcast_mut::<E>() {
            f(widget, element)
        }
    }

    /// Marks the layout tree dirty after an element updates layout-managed
    /// state that cannot be inferred from its outer [`Style`].
    #[inline]
    pub fn relayout(&mut self) {
        todo!()
    }
}

pub trait ElementSpecs<T, R: Renderer>: Any {
    fn style(&self) -> &Style;

    #[allow(unused_variables)]
    fn layout(&mut self, layout_element: &LayoutElementRef) {}

    fn mouse_event(&mut self, event: MouseEvent, layout_element: &LayoutElementRef) -> Response<T>;

    fn window_event(&mut self, event: WindowEvent, layout_element: &LayoutElementRef) -> Option<T>;

    fn input_capture(&self) -> EventKind;

    fn render(
        &self,
        layout_element: &LayoutElementRef,
        element_children: &[VisualElement<T, R>],
        render_element: RenderElement,
        renderer: &mut R,
    );
}

pub struct Element<T, R: Renderer> {
    pub(crate) children: Vec<Self>,
    pub(crate) element: Box<dyn ElementSpecs<T, R>>,
    pub(crate) layout_element: LayoutElementWeak,
}
impl<T: 'static, R: Renderer + 'static> Element<T, R> {
    pub(crate) fn layout(&mut self) {
        self.element.layout(&self.layout_element.upgrade().unwrap());

        for child in &mut self.children {
            child.layout();
        }
    }

    pub(crate) fn reconcile(&mut self, mut widget: Widget<T, R>) {
        if self.differ() == widget.key() {
            let children = widget.children();

            widget.update(self.updater());

            if let Some(layout_element) = self.layout_element.upgrade() {
                layout_element.style(|_| self.element.style().clone());
            }

            self.reconcile_children(children);
        } else {
            *self = widget.mount();
        }
    }

    fn reconcile_children(&mut self, new_children: Vec<Widget<T, R>>) {
        if self.children.len() != new_children.len() {
            self.layout_element
                .upgrade()
                .expect("This should not fail!")
                .mark_dirty();
        }

        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        }

        let mut new_children = new_children.into_iter();

        for (old, new) in self.children.iter_mut().zip(new_children.by_ref()) {
            old.reconcile(new);
        }

        self.children.extend(new_children.map(Widget::mount));
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
            layout_element: LayoutElementWeak::new(),
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
        Updater {
            element: self.element.as_mut(),
        }
    }
}
