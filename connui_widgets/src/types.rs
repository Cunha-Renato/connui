use bitflags::bitflags;
use std::sync::Arc;

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
