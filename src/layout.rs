use crate::{
    types::{Bounds, Node, Point, Position, Size, SizeOp},
    widget::Widget,
};

#[inline]
fn layout<'a, T>(root: &'a dyn Widget<T>) -> Node<'a, T> {
    resolve(root)
}

fn resolve<'a, T>(widget: &'a dyn Widget<T>) -> Node<'a, T> {
    let widget_size = widget.get_size();
    let widget_position = widget.get_position();
    let widget_layout = widget.get_layout();
    let widget_children = widget.get_children();

    todo!();
}