use std::sync::Arc;

use bitflags::bitflags;

pub enum Callback<A, R = A> {
    Fn(fn(A) -> R),
    Dyn(Arc<dyn Fn(A) -> R + Sync>),
}
impl<A, R> Callback<A, R> {
    pub const fn from_fn(f: fn(A) -> R) -> Self {
        Callback::Fn(f)
    }

    pub fn from_dyn(f: impl Fn(A) -> R + Sync + 'static) -> Self {
        Callback::Dyn(Arc::new(f))
    }

    pub fn call(&self, arg: A) -> R {
        match self {
            Callback::Fn(f) => f(arg),
            Callback::Dyn(f) => f(arg),
        }
    }
}
impl<A, R> Clone for Callback<A, R> {
    fn clone(&self) -> Self {
        match self {
            Self::Fn(f) => Self::Fn(*f),
            Self::Dyn(f) => Self::Dyn(Arc::clone(f)),
        }
    }
}
impl<A, R, F: Fn(A) -> R + Sync + 'static> From<F> for Callback<A, R> {
    fn from(f: F) -> Self {
        Self::from_dyn(f)
    }
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct WidgetStateFlags: u8 {
        const DEFAULT = 0b1;
        const HOVER = 0b10;
        const FOCUS = 0b100;
        const ACTIVE = 0b1000;
        const INACTIVE = 0b10000;
    }
}
impl WidgetStateFlags {
    pub fn set_default(&mut self) {
        self.remove(Self::ACTIVE | Self::INACTIVE);
        self.insert(Self::DEFAULT);
    }

    pub fn set_inactive(&mut self) {
        self.remove(Self::DEFAULT | Self::ACTIVE);
        self.insert(Self::INACTIVE);
    }

    pub fn set_active(&mut self) {
        self.remove(Self::DEFAULT | Self::INACTIVE);
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
