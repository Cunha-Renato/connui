use connui::{has_color, has_margin, has_positioning, tree::Style, types::*};

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(super) struct SliderState {
    pub inner: SliderInnerState,
    pub hover: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub(super) enum SliderInnerState {
    Default,
    Active,
    Inactive,
}

pub type StateFn<T> = Box<dyn FnOnce() -> T>;

pub struct SliderStates<T> {
    pub default: T,
    pub hover: Option<T>,
    pub active: Option<T>,
    pub inactive: Option<T>,
}
impl<T> SliderStates<T> {
    pub(super) fn get(self, state: SliderState) -> T {
        match state.inner {
            SliderInnerState::Default => {
                if !state.hover {
                    return self.default;
                } else {
                    self.hover
                }
            }
            SliderInnerState::Active => self.active,
            SliderInnerState::Inactive => self.inactive,
        }
        .unwrap_or(self.default)
    }
}
impl<T: Default> Default for SliderStates<T> {
    fn default() -> Self {
        Self {
            default: Default::default(),
            hover: None,
            active: None,
            inactive: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SliderStyle {
    pub(super) length: Sizing,
    pub(super) position: Positioning,
    pub(super) margin: Sides<LPixel<u16>>,
    pub(super) layout: Layout,
}
impl SliderStyle {
    pub const fn new() -> Self {
        Self {
            length: Sizing::fill_default(),
            position: Positioning::Dynamic,
            margin: Sides::all(LPixel::new(0)),
            layout: Layout::new(),
        }
    }

    /// Sets the length to [`Sizing::Fill`].
    pub fn fill(
        self,
        min_max: impl Into<MinMax<LPixel<u16>>>,
        initial: impl Into<LPixel<u16>>,
    ) -> Self {
        self.fill_const(min_max.into(), initial.into())
    }

    /// Sets the length to [`Sizing::Fill`].
    pub const fn fill_const(mut self, min_max: MinMax<LPixel<u16>>, initial: LPixel<u16>) -> Self {
        self.length = Sizing::Fill { min_max, initial };
        self
    }

    /// Sets the length to [`Sizing::Absolute`].
    pub fn absolute(self, value: impl Into<LPixel<u16>>) -> Self {
        self.absolute_const(value.into())
    }

    /// Sets the length to [`Sizing::Absolute`].
    pub const fn absolute_const(mut self, value: LPixel<u16>) -> Self {
        self.length = Sizing::Absolute(value);
        self
    }

    pub fn as_style(&self) -> Style {
        let (width, height) = self.layout.axis.horizontal_vertical(&RelativeValue {
            main: self.length,
            cross: Sizing::fit(0..=u16::MAX),
        });

        Style {
            size: Size::new(width, height),
            padding: Sides::default(),
            margin: self.margin.clone(),
            position: self.position,
            layout: self.layout,
        }
    }
}
impl Default for SliderStyle {
    fn default() -> Self {
        Self::new()
    }
}
has_positioning!(SliderStyle with {position});
has_margin!(SliderStyle with {margin});
has_layout!(SliderStyle with {layout});

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderTrackStyle {
    pub(super) thickness: Sizing,
    pub(super) color: Color,
}
impl SliderTrackStyle {
    pub const fn new() -> Self {
        Self {
            thickness: Sizing::fill_default(),
            color: Color::WHITE,
        }
    }

    /// Sets the thickness to [`Sizing::Fill`].
    pub fn fill(
        self,
        min_max: impl Into<MinMax<LPixel<u16>>>,
        initial: impl Into<LPixel<u16>>,
    ) -> Self {
        self.fill_const(min_max.into(), initial.into())
    }

    /// Sets the thickness to [`Sizing::Fill`].
    pub const fn fill_const(mut self, min_max: MinMax<LPixel<u16>>, initial: LPixel<u16>) -> Self {
        self.thickness = Sizing::Fill { min_max, initial };
        self
    }

    /// Sets the thickness to [`Sizing::Absolute`].
    pub fn absolute(self, value: impl Into<LPixel<u16>>) -> Self {
        self.absolute_const(value.into())
    }

    /// Sets the thickness to [`Sizing::Absolute`].
    pub const fn absolute_const(mut self, value: LPixel<u16>) -> Self {
        self.thickness = Sizing::Absolute(value);
        self
    }

    pub fn should_relayout(&self, other: &Self) -> bool {
        self.thickness != other.thickness
    }

    pub fn as_style(&self, main_size: Sizing, axis: LayoutAxis) -> Style {
        let size = axis.horizontal_vertical(&RelativeValue::new(main_size, self.thickness));

        Style {
            size: size.into(),
            ..Default::default()
        }
    }
}
impl Default for SliderTrackStyle {
    fn default() -> Self {
        Self::new()
    }
}
has_color!(SliderTrackStyle with {color});

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderThumbStyle {
    pub size: RelativeValue<LPixel<u16>>,
    pub color: Color,
}
impl SliderThumbStyle {
    pub const fn new() -> Self {
        Self {
            size: RelativeValue::new(LPixel::new(16), LPixel::new(16)),
            color: Color::WHITE,
        }
    }

    pub fn main(self, main: impl Into<LPixel<u16>>) -> Self {
        self.main_const(main.into())
    }

    pub const fn main_const(mut self, main: LPixel<u16>) -> Self {
        self.size.main = main;
        self
    }

    pub fn cross(self, cross: impl Into<LPixel<u16>>) -> Self {
        self.cross_const(cross.into())
    }

    pub const fn cross_const(mut self, cross: LPixel<u16>) -> Self {
        self.size.cross = cross;
        self
    }

    pub fn should_relayout(&self, other: &Self) -> bool {
        self.size != other.size
    }

    pub fn as_style(&self, axis: LayoutAxis) -> Style {
        let thickness = RelativeValue::new(0.into(), self.size.cross.into());
        let size = axis.horizontal_vertical(&thickness);

        Style {
            size: size.into(),
            ..Default::default()
        }
    }
}
impl Default for SliderThumbStyle {
    fn default() -> Self {
        Self::new()
    }
}
has_color!(SliderThumbStyle with {color});
