use crate::{
    types::{LayoutAxis, Node},
    widget::Element,
};

#[inline]
pub(crate) fn layout<'a, T>(root: &'a Element<'a, T>) -> Node<'a, T> {
    let mut node = root.into();

    resolve_size(&mut node, true);

    wrap(&mut node);

    resolve_size(&mut node, false);

    resolve_position(&mut node);

    node
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

fn wrap<'a, T>(node: &mut Node<'a, T>) {
    let widget_layout = node.widget.get_layout();

    let (along_bounds, _) = widget_layout
        .axis
        .pack(node.bounds.max.width, node.bounds.max.height);
    let mut along_offset = 0.0;
    let mut across_offset = 0.0;
    let mut along_size: f32 = 0.0;
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

        match widget_layout.axis {
            LayoutAxis::Horizontal => {
                child.position.x = along_offset;
                child.position.y = across_offset;
            }
            LayoutAxis::Vertical => {
                child.position.y = along_offset;
                child.position.x = across_offset;
            }
        }

        along_offset += along_child_size;
        new_line = new_line.max(across_offset + across_child_size);
        along_size = along_size.max(along_offset);
    }

    if widget_layout.wrap {
        let is_dynamic = node.widget.get_size().is_dynamic();

        match widget_layout.axis {
            LayoutAxis::Horizontal => {
                if is_dynamic.height {
                    node.size.height = new_line
                }
                if is_dynamic.width {
                    node.size.width = along_size;
                }
            }
            LayoutAxis::Vertical => {
                if is_dynamic.width {
                    node.size.width = new_line;
                }
                if is_dynamic.height {
                    node.size.height = along_size;
                }
            }
        }
    }
}

fn resolve_position<'a, T>(node: &mut Node<'a, T>) {
    let widget_layout = node.widget.get_layout();
    let (mut along_offset, _) = widget_layout.axis.pack(0.0, 0.0);

    for child in &mut node.children {
        // if !matches!(child.widget.get_position(), Position::Dynamic) {
        //     continue;
        // }

        if !(child.position.x != 0.0 || child.position.y != 0.0) {
            match widget_layout.axis {
                LayoutAxis::Horizontal => {
                    child.position.x += along_offset;
                    along_offset += child.size.width;
                }
                LayoutAxis::Vertical => {
                    child.position.y += along_offset;
                    along_offset += child.size.height;
                }
            }
        }

        child.position.x += node.position.x;
        child.position.y += node.position.y;

        resolve_position(child);
    }
}
