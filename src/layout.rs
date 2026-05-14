use crate::{
    types::{Node, Point, Position, Size, SizeOp},
    widget::Widget,
};

fn layout<'a, T>(root: &'a mut dyn Widget<T>) -> Node<'a, T> {
    let mut root = Node {
        children: Vec::new(),
        widget: root,
        position: Point::default(),
        size: Size::default(),
    };

    match root.widget.get_position() {
        Position::Dynamic => todo!(),
        Position::Absolute(point) | Position::Relative(point) => {
            root.position = Point {
                x: point.x as f32,
                y: point.y as f32,
            }
        }
    };

    match root.widget.get_size() {
        Size {
            width: SizeOp::Absolute(width),
            height: SizeOp::Absolute(height),
        } => {
            root.size.width = width as f32;
            root.size.height = height as f32;
        }
        Size {
            width: SizeOp::Absolute(width),
            height: _,
        } => root.size.width = width as f32,
        Size {
            width: _,
            height: SizeOp::Absolute(height),
        } => root.size.height = height as f32,
        _ => {}
    };

    root.children = root
        .widget
        .get_children()
        .iter()
        .map(|c| layout_inner(c.as_ref(), &mut root))
        .collect();

    root
}

fn layout_inner<'a, T>(current: &dyn Widget<T>, parent: &mut Node<'a, T>) -> Node<'a, T> {
    let position = Point::default();
    let mut size = Size::default();
    match current.get_size() {
        Size {
            width: SizeOp::Absolute(width),
            height: SizeOp::Absolute(height),
        } => {
            size.width = width as f32;
            size.height = height as f32;
        }
        Size {
            width: SizeOp::Absolute(width),
            height: _,
        } => size.width = width as f32,
        Size {
            width: _,
            height: SizeOp::Absolute(height),
        } => size.height = height as f32,
        _ => {}
    };

    if current.get_children().is_empty() {
        return Node {
            children: Vec::new(),
            widget: current,
            position,
            size,
        };
    }

    todo!()
}
