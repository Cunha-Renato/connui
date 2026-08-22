use std::any::TypeId;

use crate::renderer::Renderer;

use super::{Element, Updater};

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(TypeId);
impl Key {
    #[inline]
    pub const fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}
impl PartialEq<TypeId> for &Key {
    #[inline]
    fn eq(&self, other: &TypeId) -> bool {
        &self.0 == other
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
    pub fn new(widget: impl WidgetSpecs<T, R> + 'static) -> Self {
        Self(Box::new(widget))
    }

    #[inline]
    pub(super) fn key(&self) -> Key {
        self.0.key()
    }

    #[inline]
    pub(crate) fn mount(self) -> Element<T, R> {
        self.0.mount()
    }

    pub(super) fn update(self, updater: Updater) {
        self.0.update(updater);
    }

    #[inline]
    pub(super) fn children(&mut self) -> Vec<Self> {
        self.0.children()
    }
}
