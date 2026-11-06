use std::any::TypeId;

use bitflags::bitflags;

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

    fn children(&mut self) -> Vec<Widget<T, R>> {
        vec![]
    }
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

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct WidgetStateFlags: u8 {
        const DEFAULT = 0b1;
        const HOVER = 0b10;
        const FOCUS = 0b100;
        const ACTIVE = 0b1000;
        const DISABLED = 0b10000;
    }
}
impl WidgetStateFlags {
    pub fn set_default(&mut self) {
        self.remove(Self::ACTIVE | Self::DISABLED);
        self.insert(Self::DEFAULT);
    }

    pub fn set_disabled(&mut self) {
        self.remove(Self::DEFAULT | Self::ACTIVE);
        self.insert(Self::DISABLED);
    }

    pub fn set_active(&mut self) {
        self.remove(Self::DEFAULT | Self::DISABLED);
        self.insert(Self::ACTIVE);
    }

    pub fn set_hover(&mut self, hover: bool) {
        if hover {
            self.insert(Self::HOVER);
        } else {
            self.remove(Self::HOVER);
        }
    }

    pub fn set_focus(&mut self, focus: bool) {
        if focus {
            self.insert(Self::FOCUS);
        } else {
            self.remove(Self::FOCUS);
        }
    }
}
