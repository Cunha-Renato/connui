use crate::{
    types::{Bounds, LayoutAxis, Node, Point, Position, Size, SizeOp},
    widget::Widget,
};

#[inline]
pub(crate) fn layout<'a, T>(root: &'a dyn Widget<T>, bounds: Option<Bounds>) -> Node<'a, T> {
    let mut node = to_node(root, &bounds.unwrap_or_else(|| Bounds::default()));

    resolve_size(&mut node, true);

    resolve_size(&mut node, false);

    resolve_position(&mut node);

    node
}

// Converts the Element tree to a Node tree.
fn to_node<'a, T>(widget: &'a dyn Widget<T>, bounds: &Bounds) -> Node<'a, T> {
    let bounds = bounds
        .width(widget.get_size().width)
        .height(widget.get_size().height);

    let children = widget
        .get_children()
        .iter()
        .map(|c| to_node(c.as_ref(), &bounds))
        .collect();

    Node {
        children,
        widget,
        position: Default::default(),
        size: widget.get_size().as_f32(),
    }
}

fn resolve_size<'a, T>(node: &mut Node<'a, T>, along: bool) {
    node.children
        .iter_mut()
        .for_each(|c| resolve_size(c, along));

    let widget_layout = node.widget.get_layout();
    let widget_size = node.widget.get_size();
    let size_is_dynamic = {
        let is_dynamic = widget_size.is_dynamic();

        let (along_dynamic, across_dynamic) =
            widget_layout.axis.pack(is_dynamic.width, is_dynamic.height);

        if along { along_dynamic } else { across_dynamic }
    };

    if !size_is_dynamic {
        return;
    }

    let (mut along_size, mut across_size) =
        widget_layout.axis.pack(node.size.width, node.size.height);

    for child in &mut node.children {
        let (along_child_size, across_child_size) =
            widget_layout.axis.pack(child.size.width, child.size.height);

        if along {
            along_size += along_child_size;
        } else {
            across_size = across_size.max(across_child_size);
        }
    }

    match widget_layout.axis {
        LayoutAxis::Horizontal => {
            node.size.width = along_size;
            node.size.height = across_size;
        }
        LayoutAxis::Vertical => {
            node.size.width = across_size;
            node.size.height = along_size;
        }
    }
}

fn resolve_position<'a, T>(node: &mut Node<'a, T>) {
    let widget_layout = node.widget.get_layout();
    let (mut along_offset, _) = widget_layout.axis.pack(0.0, 0.0);

    node.children.iter_mut().for_each(|c| {
        match widget_layout.axis {
            LayoutAxis::Horizontal => {
                c.position.x = along_offset + node.position.x;
                along_offset += c.size.width;
            }
            LayoutAxis::Vertical => {
                c.position.y = along_offset + node.position.y;
                along_offset += c.size.height;
            }
        }

        resolve_position(c);
    });
}
