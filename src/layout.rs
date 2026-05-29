use crate::{
    types::{Node, SizeOp},
    widget::Element,
};

#[inline]
pub(crate) fn layout<'a, T>(root: &'a Element<'a, T>) -> Node<'a, T> {
    let mut node = root.into();

    resolve_fit(&mut node);
    resolve_fill(&mut node);
    wrap(&mut node);

    resolve_position(&mut node);

    node
}

fn resolve_fit<'a, T>(node: &mut Node<'a, T>) {
    node.children.iter_mut().for_each(|c| resolve_fit(c));

    let widget_layout = node.widget.get_layout();
    let (along_widget_size, across_widget_size) = widget_layout
        .axis
        .pack(node.widget.get_size().width, node.widget.get_size().height);

    if !along_widget_size.is_dynamic() && !across_widget_size.is_dynamic() {
        return;
    }

    let (along_size, across_size) = widget_layout
        .axis
        .pack(&mut node.size.width, &mut node.size.height);

    for child in &mut node.children {
        let (along_child_size, across_child_size) =
            widget_layout.axis.pack(child.size.width, child.size.height);

        if along_widget_size.is_dynamic() {
            *along_size += along_child_size;
        }
        if across_widget_size.is_dynamic() {
            *across_size = across_size.max(across_child_size);
        }
    }
}

fn resolve_fill<'a, T>(node: &mut Node<'a, T>) {
    let widget_layout = node.widget.get_layout();
    let (along_size, across_size) = widget_layout.axis.pack(node.size.width, node.size.height);

    let mut along_avail_size = along_size;
    let along_fill_children = node
        .children
        .iter_mut()
        .filter_map(|child| {
            let child_widget_size = child.widget.get_size();
            let (along_widget_size, across_widget_size) = widget_layout
                .axis
                .pack(child_widget_size.width, child_widget_size.height);
            let (along_child_size, across_child_size) = widget_layout
                .axis
                .pack(&mut child.size.width, &mut child.size.height);

            along_avail_size -= *along_child_size;

            if matches!(across_widget_size, SizeOp::Fill) {
                *across_child_size = across_size;
            }

            if matches!(along_widget_size, SizeOp::Fill) {
                Some(child)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let along_fill_children_len = along_fill_children.len() as f32;
    for along_fill_child in along_fill_children {
        let child_size = widget_layout.axis.along(
            &mut along_fill_child.size.width,
            &mut along_fill_child.size.height,
        );

        *child_size += along_avail_size / along_fill_children_len;
    }

    node.children.iter_mut().for_each(|c| resolve_fill(c));
}

fn wrap<'a, T>(node: &mut Node<'a, T>) {
    let widget_layout = node.widget.get_layout();

    let along_bounds = widget_layout
        .axis
        .along(node.bounds.max.width, node.bounds.max.height);

    let mut along_offset = 0.0;
    let mut across_offset = 0.0;
    let mut along_size_acc: f32 = 0.0;
    let mut new_line: f32 = 0.0;

    for child in &mut node.children {
        wrap(child);

        if !widget_layout.wrap {
            continue;
        }

        let (along_child_size, across_child_size) =
            widget_layout.axis.pack(child.size.width, child.size.height);

        // WRAP.
        if along_offset + along_child_size > along_bounds {
            along_offset = 0.0;
            across_offset = new_line;
        }

        let (along_child_position, across_child_position) = widget_layout
            .axis
            .pack(&mut child.position.x, &mut child.position.y);

        if along_child_size > 0.0 {
            *along_child_position = along_offset;
        }
        if across_child_size > 0.0 {
            *across_child_position = across_offset;
        }

        along_offset += along_child_size;
        new_line = new_line.max(across_offset + across_child_size);
        along_size_acc = along_size_acc.max(along_offset);
    }

    if widget_layout.wrap {
        let widget_size = node.widget.get_size();

        let (along_widget_size, across_widget_size) = widget_layout
            .axis
            .pack(widget_size.width, widget_size.height);
        let (along_size, across_size) = widget_layout
            .axis
            .pack(&mut node.size.width, &mut node.size.height);

        if along_widget_size.is_dynamic() {
            *along_size = along_size_acc;
        }

        if across_widget_size.is_dynamic() {
            *across_size = new_line;
        }
    }
}

fn resolve_position<'a, T>(node: &mut Node<'a, T>) {
    let widget_layout = node.widget.get_layout();
    let mut along_offset = 0.0;

    for child in &mut node.children {
        // if !matches!(child.widget.get_position(), Position::Dynamic) {
        //     continue;
        // }

        if !(child.position.x != 0.0 || child.position.y != 0.0) {
            let along_child_size = widget_layout
                .axis
                .along(child.size.width, child.size.height);
            let along_child_position = widget_layout
                .axis
                .along(&mut child.position.x, &mut child.position.y);

            *along_child_position += along_offset;
            along_offset += along_child_size;
        }

        child.position.x += node.position.x;
        child.position.y += node.position.y;

        resolve_position(child);
    }
}
