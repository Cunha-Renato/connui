use std::any::{Any, TypeId};

pub trait WidgetDiff: AsAny {
    #[inline]
    #[allow(unused_variables)]
    /// Returns true if **self** == **other**.
    ///
    /// **NOTE** The eq is only necessary for layout, not rendering.
    /// Diffing the [`Widget`][crate::widget::Widget]'s children is not recommended, since it can lead to unnecessary work.
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

    /// Runs meant to update [`Widget`][crate::widget::Widget] with the previous instance.
    ///
    /// **NOTE** if the [`Widget`][crate::widget::Widget] is arranged in a way that it could be reordered is recommended the
    /// use of a persistent **key** or [`Id`][crate::types::Id].
    pub fn update<T, F>(&self, other: &mut T, f: F)
    where
        T: WidgetDiff + 'static,
        F: FnOnce(&mut T, &T),
    {
        if self.tid == &TypeId::of::<T>()
            && let Some(this) = self.widget.as_any().downcast_ref::<T>()
        {
            f(other, this)
        };
    }
}
impl<'a> PartialEq for Differ<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.tid == other.tid && self.widget.diff_eq(*other)
    }
}
