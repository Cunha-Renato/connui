use crate::{
    types::{Bounds, LayoutAxis, Node, Point, Position, Size, SizeOp},
    widget::Widget,
};

#[inline]
pub(crate) fn layout<'a, T>(root: &'a dyn Widget<T>, bounds: Option<Bounds>) -> Node<'a, T> {
    let mut node = foundation(root, &bounds.unwrap_or_else(|| Bounds::default()));
    roof(&mut node, Default::default());

    node
}

// Desired sizing and position at the same time.
fn foundation<'a, T>(widget: &'a dyn Widget<T>, bounds: &Bounds) -> Node<'a, T> {
    let widget_size = widget.get_size();
    let widget_layout = widget.get_layout();
    let is_dynamic = widget_size.is_dynamic();
    let bounds = bounds.width(widget_size.width).height(widget_size.height);
    let mut size: Size = widget_size.as_f32();

    let mut children: Vec<Node<'_, T>> = widget
        .get_children()
        .iter()
        .map(|child| foundation(child.as_ref(), &bounds))
        .collect();

    // Tracks the position where the new line should be.
    let mut new_line = 0.0f32;

    let (mut along_size, mut across_size) = widget_layout.axis.pack(size.width, size.height);
    let (mut along_offset, mut across_offset) = widget_layout.axis.pack(0.0, 0.0);
    let (along_is_dynamic, across_is_dynamic) =
        widget_layout.axis.pack(is_dynamic.width, is_dynamic.height);
    let (along_bounds, _) = widget_layout.axis.pack(bounds.max.width, bounds.max.height);

    for child in &mut children {
        let (along_child_size, across_child_size) =
            widget_layout.axis.pack(child.size.width, child.size.height);
        let (along_child_pos, across_child_pos);

        // Can and should wrap.
        if widget_layout.wrap && along_offset + along_child_size >= along_bounds {
            along_offset = 0.0;
            across_offset = new_line;
        }

        // Positioning.
        along_child_pos = along_offset;
        across_child_pos = across_offset;
        along_offset += along_child_size;

        new_line = new_line.max(across_offset + across_child_size);

        // Sizing (Widget).
        if along_is_dynamic {
            along_size = along_size.max(along_offset);
        }
        if across_is_dynamic {
            across_size = across_size.max(new_line);
        }

        match widget_layout.axis {
            LayoutAxis::Horizontal => {
                child.size.width = along_child_size;
                child.size.height = across_child_size;

                child.position.x = along_child_pos;
                child.position.y = across_child_pos;

                size.width = along_size;
                size.height = across_size;
            }
            LayoutAxis::Vertical => {
                child.size.height = along_child_size;
                child.size.width = across_child_size;

                child.position.y = along_child_pos;
                child.position.x = across_child_pos;

                size.height = along_size;
                size.width = across_size;
            }
        }
    }

    match widget_layout.axis {
        LayoutAxis::Horizontal => {
            size.width = along_size;
            size.height = across_size;
        }
        LayoutAxis::Vertical => {
            size.height = along_size;
            size.width = across_size;
        }
    }

    Node {
        children,
        widget,
        position: Default::default(),
        size,
    }
}

// Resolving Shrink and Grow.
fn wall<'a, T>(node: &mut Node<'a, T>, bounds: &Bounds) {
    let widget_size = node.widget.get_size();
    let widget_layout = node.widget.get_layout();
    let is_dynamic = widget_size.is_dynamic();
    let bounds = bounds.width(widget_size.width).height(widget_size.height);

    let fill_idx: Vec<usize> = node
        .children
        .iter()
        .enumerate()
        .filter_map(|(i, child)| {
            let child_widget_size = child.widget.get_size();

            if matches!(child_widget_size.width, SizeOp::Fill)
                || matches!(child_widget_size.height, SizeOp::Fill)
            {
                Some(i)
            } else {
                None
            }
        })
        .collect();
}

// Final Position.
fn roof<'a, T>(node: &mut Node<'a, T>, position: Point) {
    // Final position.
    node.position.x += position.x;
    node.position.y += position.y;

    node.children
        .iter_mut()
        .for_each(|child| roof(child, node.position));
}
