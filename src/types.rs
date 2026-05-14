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

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum SizeOp {
    #[default]
    Fit,
    Fill,
    Absolute(u16),
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
    // TODO: Wrap
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum LayoutAxis {
    Horizontal,
    #[default]
    Vertical,
}

pub(crate) struct Node<'a, T> {
    pub children: Vec<Node<'a, T>>,
    pub widget: &'a dyn Widget<T>,
    pub position: Point,
    pub size: Size,
}
