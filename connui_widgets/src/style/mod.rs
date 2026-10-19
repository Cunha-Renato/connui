use crate::{Callback, WidgetStateFlags};

pub mod default;

pub struct StyleRules<'a, T: Clone + 'a>(std::borrow::Cow<'a, [(WidgetStateFlags, T)]>);
impl<'a, T: Clone + 'a> StyleRules<'a, T> {
    #[inline]
    pub const fn new() -> Self {
        Self(std::borrow::Cow::Borrowed(&[]))
    }

    #[inline]
    pub const fn from_slice(slice: &'a [(WidgetStateFlags, T)]) -> Self {
        Self(std::borrow::Cow::Borrowed(slice))
    }

    #[inline]
    pub const fn from_vec(vec: Vec<(WidgetStateFlags, T)>) -> Self {
        Self(std::borrow::Cow::Owned(vec))
    }

    #[inline]
    pub fn from_iter(iter: impl Iterator<Item = (WidgetStateFlags, T)>) -> Self {
        Self(std::borrow::Cow::Owned(iter.collect()))
    }

    #[inline]
    pub fn push(mut self, rule: WidgetStateFlags, value: impl Into<T>) -> Self {
        self.0.to_mut().push((rule, value.into()));

        self
    }

    #[inline]
    pub fn get(&self, flags: WidgetStateFlags) -> Option<&T> {
        self.0
            .iter()
            .find_map(|(rule, value)| rule.contains(flags).then_some(value))
    }
}
impl<'a, T: Clone + 'a> From<&'a [(WidgetStateFlags, T)]> for StyleRules<'a, T> {
    fn from(value: &'a [(WidgetStateFlags, T)]) -> Self {
        Self::from_slice(value)
    }
}
impl<'a, T: Clone + 'a> From<Vec<(WidgetStateFlags, T)>> for StyleRules<'a, T> {
    fn from(value: Vec<(WidgetStateFlags, T)>) -> Self {
        Self::from_vec(value)
    }
}

pub struct StyleSelector<'a, T: Clone> {
    base: T,
    rules: StyleRules<'a, Callback<T>>,
}
impl<'a, T: Clone> StyleSelector<'a, T> {
    #[inline]
    pub const fn new(base: T, rules: StyleRules<'a, Callback<T>>) -> Self {
        Self { base, rules }
    }

    pub fn get(&self, flags: WidgetStateFlags) -> T {
        self.rules
            .get(flags)
            .map_or_else(|| self.base.clone(), |f| f.call(self.base.clone()))
    }

    #[inline]
    pub fn base(&self) -> T {
        self.base.clone()
    }
}
