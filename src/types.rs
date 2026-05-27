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
    pub(crate) fn is_dynamic(&self) -> Size<bool> {
        Size {
            width: self.width.is_dynamic(),
            height: self.height.is_dynamic(),
        }
    }

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

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum SizeOp {
    #[default]
    Fit,
    Fill,
    Absolute(u16),
}
impl SizeOp {
    #[inline]
    pub(crate) fn is_dynamic(&self) -> bool {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub axis: LayoutAxis,
    pub overflow: bool,
    pub wrap: bool,
}
impl Default for Layout {
    #[inline]
    fn default() -> Self {
        Self {
            axis: Default::default(),
            overflow: Default::default(),
            wrap: true,
        }
    }
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

    pub(crate) fn pack<T: Copy>(&self, hor: T, ver: T) -> (T, T) {
        match self {
            LayoutAxis::Horizontal => (hor, ver),
            LayoutAxis::Vertical => (ver, hor),
        }
    }
}

// INTERNAL TYPES
#[derive(Clone, Copy)]
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

                self.min.height = new_height;
                self.max.height = new_height;
            }
            _ => {}
        }

        self
    }
}

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

    fn from_element(element: &'a Element<'a, T>, bounds: &Bounds) -> Self {
        let widget = element.as_ref();
        let bounds = bounds
            .width(widget.get_size().width)
            .height(widget.get_size().height);

        let children = widget
            .get_children()
            .iter()
            .map(|c| Self::from_element(c, &bounds))
            .collect();

        Self {
            children,
            widget,
            bounds,
            position: Default::default(),
            size: widget.get_size().as_f32(),
        }
    }
}
impl<'a, T> From<&'a Element<'a, T>> for Node<'a, T> {
    #[inline]
    fn from(element: &'a Element<'a, T>) -> Self {
        Self::from_element(element, &Default::default())
    }
}
