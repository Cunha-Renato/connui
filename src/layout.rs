use crate::{types::Node, widget::Element};

#[inline]
pub(crate) fn layout<'a, T>(root: &'a Element<'a, T>) -> Node<'a, T> {
    let mut node = root.into();

    resolve_fit(&mut node, true);

    wrap(&mut node);

    resolve_fit(&mut node, false);

    resolve_position(&mut node);

    node
}

fn resolve_fit<'a, T>(node: &mut Node<'a, T>, along: bool) {
    node.children.iter_mut().for_each(|c| resolve_fit(c, along));

    let widget_layout = node.widget.get_layout();
    let (along_widget_size, across_widget_size) = widget_layout
        .axis
        .pack(node.widget.get_size().width, node.widget.get_size().height);

    let widget_size = if along {
        along_widget_size
    } else {
        across_widget_size
    };

    match widget_size {
        crate::types::SizeOp::Absolute(_) => return,
        _ => {}
    }

    let (along_size, across_size) = widget_layout
        .axis
        .pack(&mut node.size.width, &mut node.size.height);

    for child in &mut node.children {
        let (along_child_size, across_child_size) =
            widget_layout.axis.pack(child.size.width, child.size.height);

        if along {
            *along_size += along_child_size;
        } else {
            *across_size = across_size.max(across_child_size);
        }
    }
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

        *along_child_position = along_offset;
        *across_child_position = across_offset;

        along_offset += along_child_size;
        new_line = new_line.max(across_offset + across_child_size);
        along_size_acc = along_size_acc.max(along_offset);
    }

    if widget_layout.wrap {
        let is_dynamic = node.widget.get_size().is_dynamic();

        let (along_dynamic, across_dynamic) =
            widget_layout.axis.pack(is_dynamic.width, is_dynamic.height);
        let (along_size, across_size) = widget_layout
            .axis
            .pack(&mut node.size.width, &mut node.size.height);

        if along_dynamic {
            *along_size = along_size_acc;
        }
        if across_dynamic {
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
