use crate::{renderer::Renderer, types::*};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct Handle<R: Renderer> {
    id: Id,
    load: Arc<HandleLoad<R>>,
}
impl<R: Renderer> Handle<R> {
    #[inline]
    pub fn once<F: FnOnce() -> HandleKind<R> + Send + 'static>(id: Id, f: F) -> Self {
        Self {
            id,
            load: Arc::new(HandleLoad::Once(f.into())),
        }
    }

    #[inline]
    pub fn always<F: FnOnce() -> HandleKind<R> + Send + 'static>(id: Id, f: F) -> Self {
        Self {
            id,
            load: Arc::new(HandleLoad::Always(f.into())),
        }
    }

    #[inline]
    pub fn id(&self) -> Id {
        self.id
    }

    #[inline]
    pub fn load(&self) -> &HandleLoad<R> {
        &self.load
    }
}
impl<R: Renderer> Clone for Handle<R> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            load: Arc::clone(&self.load),
        }
    }
}

pub enum HandleKind<R: Renderer> {
    Path(PathBuf),
    Bytes(Box<[u8]>),
    Custom {
        width: u32,
        height: u32,
        bpp: u32,
        bytes: Box<[u8]>,
    },
    Gpu(R::ImageHandle),
}

pub enum HandleLoad<R: Renderer> {
    Once(HandleContentCallback<HandleKind<R>>),
    Always(HandleContentCallback<HandleKind<R>>),
}
impl<R: Renderer> HandleLoad<R> {
    #[inline]
    pub fn take(&self) -> Option<HandleKind<R>> {
        match self {
            Self::Once(callback) => callback.take(),
            Self::Always(callback) => callback.take(),
        }
    }
}
impl<R: Renderer> Clone for HandleLoad<R> {
    fn clone(&self) -> Self {
        match self {
            Self::Once(arg0) => Self::Once(arg0.clone()),
            Self::Always(arg0) => Self::Always(arg0.clone()),
        }
    }
}

/// Represents a run-once callback, cheap to clone, safely consumable from
/// any clone exactly once.
#[allow(clippy::type_complexity)]
pub struct HandleContentCallback<T>(Arc<Mutex<Option<Box<dyn FnOnce() -> T + Send>>>>);
impl<T> HandleContentCallback<T> {
    /// Runs and consumes the inner closure, returning [`Some`]. Every
    /// subsequent call, from this handle or any clone, returns [`None`].
    #[inline]
    pub fn take(&self) -> Option<T> {
        // TODO: Handle poison error (see catch_unwind approach if this becomes an issue).
        self.0.lock().ok()?.take().map(|f| f())
    }
}
impl<T> Clone for HandleContentCallback<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
impl<T, F> From<F> for HandleContentCallback<T>
where
    F: FnOnce() -> T + Send + 'static,
{
    fn from(value: F) -> Self {
        Self(Arc::new(Mutex::new(Some(Box::new(value)))))
    }
}

/// Must have 4 bytes per pixel.
#[derive(Debug, Clone)]
pub struct WriteOp<'a> {
    pub rect: Rect<u32, u32>,
    pub bytes: &'a [u8],
}
