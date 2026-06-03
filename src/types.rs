use crate::widget::{Element, Widget};

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
    pub(crate) fn as_f32(&self) -> Size<f32> {
        Size {
            width: match self.width {
                SizeOp::Absolute(width) => width as f32,
                _ => 0.0,
            },
            height: match self.height {
                SizeOp::Absolute(height) => height as f32,
                _ => 0.0,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeOp {
    Fit { min: u16, max: u16 },
    Fill { min: u16, max: u16 },
    Absolute(u16),
}
impl SizeOp {
    #[inline]
    pub const fn fit() -> Self {
        Self::Fit {
            min: 0,
            max: u16::MAX,
        }
    }

    #[inline]
    pub const fn fill() -> Self {
        Self::Fill {
            min: 0,
            max: u16::MAX,
        }
    }

    #[inline]
    pub const fn absolute(value: u16) -> Self {
        Self::Absolute(value)
    }

    #[inline]
    pub(crate) const fn is_dynamic(&self) -> bool {
        matches!(self, SizeOp::Fit { .. } | SizeOp::Fill { .. })
    }
}
impl Default for SizeOp {
    #[inline]
    fn default() -> Self {
        Self::fit()
    }
}
impl From<u16> for SizeOp {
    #[inline]
    fn from(value: u16) -> Self {
        Self::Absolute(value)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Position {
    #[default]
    Dynamic,
    Absolute(Point<i16>),
    Relative(Point<i16>),
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Sides<T = f32> {
    pub top: T,
    pub bottom: T,
    pub right: T,
    pub left: T,
}
impl<T> Sides<T> {
    pub const fn all(value: T) -> Self
    where
        T: Copy,
    {
        Self {
            top: value,
            bottom: value,
            right: value,
            left: value,
        }
    }

    #[inline]
    pub fn horizontal(&self) -> T
    where
        T: std::ops::Add<Output = T> + Copy,
    {
        self.left + self.right
    }

    #[inline]
    pub fn vertical(&self) -> T
    where
        T: std::ops::Add<Output = T> + Copy,
    {
        self.top + self.bottom
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub axis: LayoutAxis,
    pub overflow: bool,
    pub wrap: bool,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum LayoutAxis {
    #[default]
    Horizontal,
    Vertical,
}
impl LayoutAxis {
    #[inline]
    pub(crate) fn along<T>(&self, hor: T, ver: T) -> T {
        match self {
            LayoutAxis::Horizontal => hor,
            LayoutAxis::Vertical => ver,
        }
    }

    #[inline]
    pub(crate) fn pack<T>(&self, hor: T, ver: T) -> (T, T) {
        match self {
            LayoutAxis::Horizontal => (hor, ver),
            LayoutAxis::Vertical => (ver, hor),
        }
    }
}

pub enum Response<T> {
    Value(T),
    Callback(Box<dyn FnOnce() -> T>),
}

// INTERNAL

#[derive(Debug, Clone, Copy)]
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
    pub fn width(mut self, width: SizeOp, padding: u16) -> Self {
        match width {
            SizeOp::Fit { min, max } | SizeOp::Fill { min, max } => {
                self.min.width = min as f32;
                self.max.width = (self.max.width - padding as f32)
                    .min(max as f32)
                    .max(min as f32);
            }
            SizeOp::Absolute(width) => {
                let new_width = width as f32;

                self.min.width = new_width;
                self.max.width = new_width;
            }
        }

        self
    }

    pub fn height(mut self, height: SizeOp, padding: u16) -> Self {
        match height {
            SizeOp::Fit { min, max } | SizeOp::Fill { min, max } => {
                self.min.height = min as f32;
                self.max.height = (self.max.height - padding as f32)
                    .min(max as f32)
                    .max(min as f32);
            }
            SizeOp::Absolute(height) => {
                let new_height = height as f32;

                self.min.height = new_height;
                self.max.height = new_height;
            }
        }

        self
    }
}

#[derive(Clone)]
pub(crate) struct Node<'a, T> {
    pub children: Vec<Node<'a, T>>,
    pub widget: &'a dyn Widget<T>,
    pub bounds: Bounds,
    pub position: Point,
    pub size: Size,
}
impl<T> std::fmt::Debug for Node<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("widget_id", &self.widget.get_id())
            .field("position", &self.position)
            .field("size", &self.size)
            .field("children", &self.children)
            .finish()
    }
}
impl<'a, T> Node<'a, T> {
    pub fn render(&self) -> Vec<crate::renderer::RenderCommand> {
        let mut commands = Vec::new();

        // Render self.
        commands.extend(self.widget.render(self.position, self.size));

        // Render children.
        for child in &self.children {
            commands.extend(child.render());
        }

        commands
    }

    pub fn from_widget(widget: &'a dyn Widget<T>, bounds: &Bounds, padding: Sides<u16>) -> Self {
        let bounds = bounds
            .width(widget.get_size().width, padding.horizontal())
            .height(widget.get_size().height, padding.vertical());

        let children = widget
            .get_children()
            .iter()
            .map(|c| Self::from_element(c, &bounds, widget.get_padding()))
            .collect();

        Self {
            children,
            widget,
            bounds,
            position: Default::default(),
            size: widget.get_size().as_f32(),
        }
    }

    #[inline]
    pub fn from_element(element: &'a Element<'a, T>, bounds: &Bounds, padding: Sides<u16>) -> Self {
        let widget = element.as_ref();
        Self::from_widget(widget, bounds, padding)
    }
}
impl<'a, T> From<&'a Element<'a, T>> for Node<'a, T> {
    #[inline]
    fn from(element: &'a Element<'a, T>) -> Self {
        Self::from_element(element, &Default::default(), Default::default())
    }
}
impl<'a, T> From<&'a dyn Widget<T>> for Node<'a, T> {
    #[inline]
    fn from(widget: &'a dyn Widget<T>) -> Self {
        Self::from_widget(widget, &Default::default(), Default::default())
    }
}
