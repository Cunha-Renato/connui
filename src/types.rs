use crate::widget::Widget;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Point<T = f32> {
    pub x: T,
    pub y: T,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Size<T = f32> {
    pub width: T,
    pub height: T,
}
impl Size<SizeOp> {
    pub(crate) fn should_shrink(&self) -> Size<bool> {
        Size {
            width: self.width.should_shrink(),
            height: self.height.should_shrink(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum SizeOp {
    #[default]
    Fit,
    Fill,
    Absolute(u16),
}
impl SizeOp {
    #[inline]
    pub(crate) fn should_shrink(&self) -> bool {
        matches!(self, SizeOp::Fit | SizeOp::Fill)
    }
}

pub enum Response<T> {
    Value(T),
    Callback(Box<dyn FnOnce() -> T>),
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Position {
    #[default]
    Dynamic,
    Absolute(Point<i16>),
    Relative(Point<i16>),
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub axis: LayoutAxis,
    pub wrap: bool,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum LayoutAxis {
    Horizontal,
    #[default]
    Vertical,
}
impl LayoutAxis {
    pub(crate) fn along(&self, size: Size) -> f32 {
        match self {
            LayoutAxis::Horizontal => size.width,
            LayoutAxis::Vertical => size.height,
        }
    }

    pub(crate) fn across(&self, size: Size) -> f32 {
        match self {
            LayoutAxis::Horizontal => size.height,
            LayoutAxis::Vertical => size.width,
        }
    }

    pub(crate) fn pack<T>(&self, size: Size<T>) -> (T, T) {
        match self {
            LayoutAxis::Horizontal => (size.width, size.height),
            LayoutAxis::Vertical => (size.height, size.width),
        }
    }
}

// INTERNAL TYPES

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Bounds {
    pub min: Size,
    pub max: Size,
}
impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: Size {
                width: 0.0,
                height: 0.0,
            },
            max: Size {
                width: f32::INFINITY,
                height: f32::INFINITY,
            },
        }
    }
}
impl Bounds {
    pub fn width(mut self, width: SizeOp) -> Self {
        match width {
            SizeOp::Absolute(width) => {
                let new_width = (width as f32).min(self.max.width).max(self.min.width);

                self.min.width = new_width;
                self.max.width = new_width;
            }
            _ => {}
        }

        self
    }

    pub fn height(mut self, height: SizeOp) -> Self {
        match height {
            SizeOp::Absolute(height) => {
                let new_height = (height as f32).min(self.max.height).max(self.min.height);

                self.min.width = new_height;
                self.max.width = new_height;
            }
            _ => {}
        }

        self
    }

    #[inline]
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min.width = self.min.width.max(min_width).min(self.max.width);

        self
    }

    #[inline]
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min.height = self.min.height.max(min_height).min(self.max.height);

        self
    }

    #[inline]
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max.width = self.max.width.min(max_width).max(self.min.width);

        self
    }

    #[inline]
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max.height = self.max.height.min(max_height).max(self.min.height);

        self
    }

    pub fn resolve(&self, width: SizeOp, height: SizeOp, intrinsic_size: Size) -> Size {
        let width = match width {
            SizeOp::Fill => self.max.width,
            SizeOp::Absolute(width) => (width as f32).min(self.max.width).max(self.min.width),
            _ => intrinsic_size.width.min(self.max.width).max(self.min.width),
        };

        let height = match height {
            SizeOp::Fill => self.max.height,
            SizeOp::Absolute(height) => (height as f32).min(self.max.height).max(self.min.height),
            _ => intrinsic_size
                .height
                .min(self.max.height)
                .max(self.min.height),
        };

        Size { width, height }
    }
}

pub(crate) struct Node<'a, T> {
    pub children: Vec<Node<'a, T>>,
    pub widget: &'a dyn Widget<T>,
    pub position: Point,
    pub size: Size,
}
