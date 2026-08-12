use std::any::{Any, TypeId};

pub trait WidgetDiff: AsAny {
    #[inline]
    #[allow(unused_variables)]
    /// Returns true if **self** == **other**.
    ///
    /// **NOTE** The eq is only necessary for layout, not rendering.
    fn diff_eq(&self, other: Differ) -> bool {
        false
    }
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}
impl<T: Any> AsAny for T {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Copy)]
pub struct Differ<'a> {
    widget: &'a dyn WidgetDiff,
    tid: &'a TypeId,
}
impl<'a> Differ<'a> {
    #[inline]
    pub(super) fn new(widget: &'a dyn WidgetDiff, tid: &'a TypeId) -> Self {
        Self { widget, tid }
    }

    /// Checks if the type is the same, if so run **f**.
    pub fn diff_eq<T, F>(&self, other: &T, f: F) -> bool
    where
        T: WidgetDiff + 'static,
        F: FnOnce(&T, &T) -> bool,
    {
        if self.tid == &TypeId::of::<T>() {
            let Some(this) = self.widget.as_any().downcast_ref::<T>() else {
                return false;
            };

            return f(other, this);
        }

        false
    }
}
impl<'a> PartialEq for Differ<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.tid == other.tid && self.widget.diff_eq(*other)
    }
}

pub(crate) struct DiffElement {
    widget: Box<dyn WidgetDiff>,
    tid: std::any::TypeId,
}
impl DiffElement {
    #[inline]
    pub(super) fn differ(&self) -> Differ<'_> {
        Differ {
            widget: self.widget.as_ref(),
            tid: &self.tid,
        }
    }
}
impl PartialEq for DiffElement {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.tid == other.tid && self.widget.diff_eq(other.differ())
    }
}
impl<T, R: crate::renderer::Renderer> From<super::Element<T, R>> for DiffElement {
    fn from(value: super::Element<T, R>) -> Self {
        Self {
            widget: value.widget,
            tid: value.tid,
        }
    }
}
