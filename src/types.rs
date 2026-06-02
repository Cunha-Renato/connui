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
impl From<u16> for SizeOp {
    #[inline]
    fn from(value: u16) -> Self {
        Self::Absolute(value)
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
    pub(crate) const fn pack<T>(&self, hor: T, ver: T) -> (T, T) {
        match self {
            LayoutAxis::Horizontal => (hor, ver),
            LayoutAxis::Vertical => (ver, hor),
        }
    }
}

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
    pub fn width(mut self, width: SizeOp) -> Self {
        match width {
            SizeOp::Fit | SizeOp::Fill => self.min.width = 0.0,
            SizeOp::Absolute(width) => {
                let new_width = width as f32;

                self.min.width = new_width;
                self.max.width = new_width;
            }
        }

        self
    }

    pub fn height(mut self, height: SizeOp) -> Self {
        match height {
            SizeOp::Fit | SizeOp::Fill => self.min.height = 0.0,
            SizeOp::Absolute(height) => {
                let new_height = height as f32;

                self.min.height = new_height;
                self.max.height = new_height;
            }
        }

        self
    }
}

pub(crate) enum NodeWidget<'a, T> {
    Ref(&'a (dyn Widget<T> + 'a)),
    Own(Element<'a, T>),
}
impl<'a, T> std::ops::Deref for NodeWidget<'a, T> {
    type Target = dyn Widget<T> + 'a;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            NodeWidget::Ref(widget) => *widget,
            NodeWidget::Own(element) => element.as_ref(),
        }
    }
}
impl<'a, T> From<&'a dyn Widget<T>> for NodeWidget<'a, T> {
    #[inline]
    fn from(value: &'a dyn Widget<T>) -> Self {
        Self::Ref(value)
    }
}
impl<'a, T> From<Element<'a, T>> for NodeWidget<'a, T> {
    #[inline]
    fn from(value: Element<'a, T>) -> Self {
        Self::Own(value)
    }
}

pub(crate) struct Node<'a, T> {
    pub widget: NodeWidget<'a, T>,
    pub children: Vec<Node<'a, T>>,
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

    pub fn from_widget(widget: &'a dyn Widget<T>, bounds: &Bounds) -> Self {
        let bounds = bounds
            .width(widget.get_size().width)
            .height(widget.get_size().height);

        let children = widget
            .get_children()
            .iter()
            .map(|c| Self::from_ref_element(c, &bounds))
            .collect();

        Self {
            children,
            widget: widget.into(),
            bounds,
            position: Default::default(),
            size: widget.get_size().as_f32(),
        }
    }

    #[inline]
    pub fn from_ref_element(element: &'a Element<'a, T>, bounds: &Bounds) -> Self {
        let widget = element.as_ref();
        Self::from_widget(widget, bounds)
    }

    #[inline]
    pub fn from_element(element: Element<'a, T>, bounds: &Bounds) -> Self {
        let bounds = bounds
            .width(element.get_size().width)
            .height(element.get_size().height);
        let size = element.get_size().as_f32();

        Self {
            widget: element.into(),
            children: vec![],
            bounds,
            position: Default::default(),
            size,
        }
    }
}
impl<'a, T> From<&'a Element<'a, T>> for Node<'a, T> {
    #[inline]
    fn from(element: &'a Element<'a, T>) -> Self {
        Self::from_ref_element(element, &Default::default())
    }
}
impl<'a, T> From<&'a dyn Widget<T>> for Node<'a, T> {
    #[inline]
    fn from(widget: &'a dyn Widget<T>) -> Self {
        Self::from_widget(widget, &Default::default())
    }
}
