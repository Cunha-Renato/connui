use std::any::{Any, TypeId};

use slotmap::{SlotMap, new_key_type};

use crate::{event::Event, renderer::Renderer, types::*};

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(TypeId);
impl Key {
    #[inline]
    pub const fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}

#[repr(transparent)]
struct Differ<'a>(&'a dyn Any);
impl<'a> Differ<'a> {
    #[inline]
    fn diff(self, key: Key) -> bool {
        self.0.type_id() == key.0
    }
}

#[repr(transparent)]
pub struct Updater<'a>(&'a mut dyn Any);
impl<'a> Updater<'a> {
    #[inline]
    pub fn update<W: 'static, E, F>(self, element: &mut E, f: F)
    where
        F: FnOnce(&mut E, &mut W),
    {
        if let Some(widget) = self.0.downcast_mut::<W>() {
            f(element, widget)
        }
    }
}

pub trait WidgetSpecs<T, R: Renderer> {
    fn key(&self) -> Key;

    fn mount(self: Box<Self>) -> Element<T, R>;

    fn update(self: Box<Self>, updater: Updater);

    fn children(&mut self) -> Vec<Widget<T, R>>;
}

#[repr(transparent)]
pub struct Widget<T, R: Renderer>(Box<dyn WidgetSpecs<T, R>>);
impl<T, R: Renderer> Widget<T, R> {
    #[inline]
    fn key(&self) -> Key {
        self.0.key()
    }

    #[inline]
    fn mount(self) -> Element<T, R> {
        self.0.mount()
    }

    fn update(self, updater: Updater) {
        self.0.update(updater);
    }

    #[inline]
    fn children(&mut self) -> Vec<Self> {
        self.0.children()
    }
}

pub trait WidgetElementSpecs<T, R: Renderer>: Any {
    fn event(&mut self, event: Event) -> Response<T>;

    fn render(&self, renderer: &mut R);
}

pub struct Element<T, R: Renderer> {
    children: Vec<Self>,
    element: Box<dyn WidgetElementSpecs<T, R>>,
}
impl<T, R: Renderer> Element<T, R> {
    #[inline]
    pub fn new<W, E>(widget: &mut W, element: E) -> Self
    where
        W: WidgetSpecs<T, R>,
        E: WidgetElementSpecs<T, R> + 'static,
    {
        Self {
            children: widget.children().into_iter().map(Widget::mount).collect(),
            element: Box::new(element),
        }
    }

    pub fn render(&self, renderer: &mut R) {
        todo!()
    }

    pub(crate) fn event(&mut self, event: Event) {
        todo!()
    }

    pub(crate) fn reconcile(&mut self, mut widget: Widget<T, R>) {
        if self.differ().diff(widget.key()) {
            let children = widget.children();

            widget.update(self.updater());
            self.reconcile_children(children);
        } else {
            *self = widget.mount();
        }
    }

    fn reconcile_children(&mut self, new_children: Vec<Widget<T, R>>) {
        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        }

        let mut new_children = new_children.into_iter();

        for (old, new) in self.children.iter_mut().zip(new_children.by_ref()) {
            old.reconcile(new);
        }

        self.children.extend(new_children.map(Widget::mount));
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
