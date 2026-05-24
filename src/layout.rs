use crate::{
    types::{Bounds, LayoutAxis, Node, Point, Position, Size, SizeOp},
    widget::Widget,
};

#[inline]
fn layout<'a, T>(root: &'a dyn Widget<T>, bounds: Option<Bounds>) -> Node<'a, T> {
    resolve(root, &bounds.unwrap_or_else(|| Bounds::default()))
}

fn resolve<'a, T>(widget: &'a dyn Widget<T>, bounds: &Bounds) -> Node<'a, T> {
    let widget_size = widget.get_size();
    let widget_layout = widget.get_layout();
    let is_dynamic = widget_size.is_dynamic();
    let bounds = bounds.width(widget_size.width).height(widget_size.height);

    let mut size: Size = Size::default();
    let mut position = Point::default();

    let mut children: Vec<Node<'_, T>> = widget
        .get_children()
        .iter()
        .map(|child| resolve(child.as_ref(), &bounds))
        .collect();

    // First Pass. (Fit and Fill)
    if is_dynamic.width || is_dynamic.height {
        children = children
            .into_iter()
            .filter(|child| {
                let (along, across) = widget_layout.axis.pack(child.size);

                // Growing.
                match widget_layout.axis {
                    LayoutAxis::Horizontal => {
                        if is_dynamic.width {
                            size.width += along;
                        } else {
                            size.height = size.height.max(across);
                        }
                    }
                    LayoutAxis::Vertical => {
                        if is_dynamic.height {
                            size.height += along;
                        } else {
                            size.width = size.width.max(across);
                        }
                    }
                }

                // Overflow.
                // Early exit if we have already filled the bounds.
                !(!widget_layout.overflow
                    && (size.height >= bounds.max.height || size.width >= bounds.max.width))
            })
            .collect();
    }

    // Final Pass. (Positioning)
    let mut offset = 0.0;
    for child in &mut children {
        let (along, _) = widget_layout.axis.pack(child.size);
        let (bounds_along_min, _) = widget_layout.axis.pack(bounds.min);

        match widget_layout.axis {
            LayoutAxis::Horizontal => {
                child.position.x = position.x + offset;
                child.position.y = position.y;
            }
            LayoutAxis::Vertical => {
                child.position.x = position.x;
                child.position.y = position.y + offset;
            }
        }

        offset += along;

        // Overflow.
        // Early exit if we have already exited the bounds.
        if !widget_layout.overflow && offset >= bounds_along_min + along {
            break;
        }
    }

    // Clamp to bounds.
    size.width = size.width.min(bounds.max.width);
    size.height = size.height.min(bounds.max.height);

    Node {
        children,
        widget,
        position,
        size,
    }
}
