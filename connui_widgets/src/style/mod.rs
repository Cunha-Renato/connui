use crate::{Callback, WidgetStateFlags};

pub mod default;

pub struct StyleRules<'a, T>(std::borrow::Cow<'a, [(WidgetStateFlags, Callback<T>)]>);
impl<'a, T: Clone> StyleRules<'a, T> {
    #[inline]
    pub const fn new() -> Self {
        Self(std::borrow::Cow::Borrowed(&[]))
    }

    #[inline]
    pub const fn from_slice(slice: &'a [(WidgetStateFlags, Callback<T>)]) -> Self {
        Self(std::borrow::Cow::Borrowed(slice))
    }

    #[inline]
    pub const fn from_vec(vec: Vec<(WidgetStateFlags, Callback<T>)>) -> Self {
        Self(std::borrow::Cow::Owned(vec))
    }

    #[inline]
    pub fn from_iter(iter: impl Iterator<Item = (WidgetStateFlags, Callback<T>)>) -> Self {
        Self(std::borrow::Cow::Owned(iter.collect()))
    }

    #[inline]
    pub fn push<F: Fn(T) -> T + 'static + Sync>(
        mut self,
        rule: WidgetStateFlags,
        value: F,
    ) -> Self {
        self.0.to_mut().push((rule, Callback::from_dyn(value)));

        self
    }

    #[inline]
    pub fn get(&self, flags: WidgetStateFlags) -> Option<&Callback<T>> {
        self.0
            .iter()
            .find_map(|(rule, f)| rule.contains(flags).then_some(f))
    }
}
impl<'a, T: Clone> From<&'a [(WidgetStateFlags, Callback<T>)]> for StyleRules<'a, T> {
    fn from(value: &'a [(WidgetStateFlags, Callback<T>)]) -> Self {
        Self::from_slice(value)
    }
}
impl<'a, T: Clone> From<Vec<(WidgetStateFlags, Callback<T>)>> for StyleRules<'a, T> {
    fn from(value: Vec<(WidgetStateFlags, Callback<T>)>) -> Self {
        Self::from_vec(value)
    }
}

pub struct StyleSelector<'a, T: Clone> {
    pub base: T,
    pub rules: StyleRules<'a, T>,
}
impl<'a, T: Clone> StyleSelector<'a, T> {
    #[inline]
    pub const fn new(base: T, rules: StyleRules<'a, T>) -> Self {
        Self { base, rules }
    }

    pub fn get(&self, flags: WidgetStateFlags) -> T {
        self.rules
            .get(flags)
            .or_else(|| self.rules.get(WidgetStateFlags::DEFAULT))
            .map_or_else(|| self.base.clone(), |f| f.call(self.base.clone()))
    }
}
