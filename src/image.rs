use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{renderer::Renderer, types::Id};

/// Handle to an image / texture, works by having an [`Id`] and a [`HandleContentCallback`]. It is cheap to clone.
///
/// This allows to cache the [`HandleContent`] by running the closure only when it wasn't initialized or when the image changes.
#[derive(Clone)]
pub struct Handle<R: Renderer> {
    id: Id,
    content: HandleContentCallback<HandleContent<R>>,
}
impl<R: Renderer + 'static> Handle<R> {
    #[inline]
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::_new(None, path.into())
    }

    #[inline]
    pub fn callback<C>(id: Id, content: C) -> Self
    where
        C: Into<HandleContentCallback<HandleContent<R>>>,
    {
        Self {
            id,
            content: content.into(),
        }
    }

    #[inline]
    pub fn new<C: Into<HandleContent<R>>>(id: Id, content: C) -> Self {
        Self::_new(Some(id), content)
    }

    #[inline]
    pub fn id(&self) -> Id {
        self.id
    }

    #[inline]
    pub fn take(&self) -> Option<HandleContent<R>> {
        self.content.take()
    }

    fn _new<C: Into<HandleContent<R>>>(id: Option<Id>, content: C) -> Self {
        let content = content.into();
        let id = match &content {
            HandleContent::Path(path) => {
                if let Some(id) = id {
                    id
                } else {
                    Id::new(path)
                }
            }
            // Safety: This is only called internaly, with 100% sure the id is Some here.
            HandleContent::Bytes(_) | HandleContent::Gpu(_) | HandleContent::Custom(_) => {
                id.unwrap()
            }
        };

        Self::callback(id, || content)
    }
}

/// Type of content held by the [`Handle`].
pub enum HandleContent<R: Renderer> {
    Path(PathBuf),
    Bytes(Vec<u8>),
    Gpu(R::ImageHandle),
    Custom(CustomHandleContent),
}
impl<R: Renderer> From<PathBuf> for HandleContent<R> {
    #[inline]
    fn from(value: PathBuf) -> Self {
        Self::Path(value)
    }
}
impl<R: Renderer> From<Vec<u8>> for HandleContent<R> {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}
impl<R: Renderer> From<CustomHandleContent> for HandleContent<R> {
    #[inline]
    fn from(value: CustomHandleContent) -> Self {
        Self::Custom(value)
    }
}

/// Represents a run once callback.
#[allow(clippy::type_complexity)]
pub struct HandleContentCallback<T>(Arc<Mutex<Option<Box<dyn FnOnce() -> T + Send>>>>);
impl<T> HandleContentCallback<T> {
    /// Will run and consume the inner closure, returning [`Some`]. Subsequent calls will return [`None`].
    #[inline]
    pub fn take(&self) -> Option<T> {
        // TODO: Handle poison error.
        self.0.lock().ok()?.take().map(|f| f())
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
impl<T> Clone for HandleContentCallback<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[derive(Clone)]
pub struct CustomHandleContent(Arc<dyn CustomHandleContentSpecs + Send + Sync>);
impl std::ops::Deref for CustomHandleContent {
    type Target = Arc<dyn CustomHandleContentSpecs + Send + Sync>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: CustomHandleContentSpecs + 'static> From<T> for CustomHandleContent {
    #[inline]
    fn from(value: T) -> Self {
        Self(Arc::new(value))
    }
}
impl From<Arc<dyn CustomHandleContentSpecs>> for CustomHandleContent {
    #[inline]
    fn from(value: Arc<dyn CustomHandleContentSpecs>) -> Self {
        Self(value)
    }
}

pub trait CustomHandleContentSpecs: Send + Sync {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn bytes(&self) -> &[u8];
    fn bytes_per_pixel(&self) -> u32;
}
