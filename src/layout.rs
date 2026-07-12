use crate::{
    renderer::Renderer,
    types::{LayoutAxis, LayoutFlags, Node, Point, Position, Rect, Size, SizeOp},
    widget::{Element, Widget},
};

pub(crate) fn layout<T, R: Renderer + 'static>(node: &mut Node<T, R>, scale_factor: f32) {
    resolve_fit(node);
    resolve_fill(node);

    // This was the stable way I found to make wrapping decent.
    if wrap(node) {
        resolve_fit(node);
        resolve_fill(node);
    }

    let mut overlay = Vec::new();
    resolve_position(node, &mut overlay, None);
    node.children.extend(overlay);

    resolve_scaling(node, scale_factor);
}

fn resolve_fit<T, R: Renderer + 'static>(node: &mut Node<T, R>) {
    // Children first.
    node.children.iter_mut().for_each(resolve_fit);

    let axis = node.widget.get_layout().axis;
    let (along_widget_size, across_widget_size) = axis.pack(&node.widget.get_size());

    // Absolute size, no need for this pass.
    if !along_widget_size.is_dynamic() && !across_widget_size.is_dynamic() {
        return;
    }

    // Initialize the size.
    node.size = node.widget.get_size().as_f32();
    let (along_size, across_size) = axis.pack_mut(&mut node.size);

    // Fit children.
    for child in &mut node.children {
        // Skip if parent should ignore.
        if child
            .widget
            .get_layout()
            .flags
            .contains(LayoutFlags::PARENT_IGNORE)
        {
            continue;
        }

        let child_margin = child.widget.get_margin();
        let (along_child_size, across_child_size) = axis.pack(&(
            child.size.width + child_margin.get_horizontal() as f32,
            child.size.height + child_margin.get_vertical() as f32,
        ));

        if along_widget_size.is_dynamic() {
            *along_size += along_child_size;
        }
        if across_widget_size.is_dynamic() {
            *across_size = across_size.max(across_child_size);
        }
    }

    // Padding.
    let padding = node.widget.get_padding();
    node.size.width += padding.get_horizontal() as f32;
    node.size.height += padding.get_vertical() as f32;

    // Clamp to bounds.
    node.size.width = node
        .size
        .width
        .clamp(node.bounds.min.width, node.bounds.max.width);
    node.size.height = node
        .size
        .height
        .clamp(node.bounds.min.height, node.bounds.max.height);
}

fn resolve_fill<T, R: Renderer + 'static>(node: &mut Node<T, R>) {
    let axis = node.widget.get_layout().axis;

    let (along_padding, across_padding) = axis.pack(&node.widget.get_padding());
    let (along_size, across_size) = axis.pack(&node.size);

    let (mut along_content, across_content) = (
        along_size - along_padding as f32,
        across_size - across_padding as f32,
    );

    for child in &mut node.children {
        // Skip if parent should ignore.
        if child
            .widget
            .get_layout()
            .flags
            .contains(LayoutFlags::PARENT_IGNORE)
        {
            continue;
        }

        let (along_child_size, _) = axis.pack(&child.size);
        let along_margin = axis.along(&child.widget.get_margin()) as f32;
        along_content -= along_child_size + along_margin;

        // Across fill.
        let across_widget_size = axis.across(&child.widget.get_size());
        if matches!(across_widget_size, SizeOp::Fill { .. }) {
            let across_margin = axis.across(&child.widget.get_margin()) as f32;
            *axis.across_mut(&mut child.size) = (across_content - across_margin).max(0.0);
        }
    }

    // RESIZING
    // Grow
    if along_content > 0.0 {
        // Along fill children.
        let mut fill_children: Vec<_> = node
            .children
            .iter_mut()
            .filter(|child| {
                // Skip if parent should ignore.
                if child
                    .widget
                    .get_layout()
                    .flags
                    .contains(LayoutFlags::PARENT_IGNORE)
                {
                    return false;
                }

                let (along_widget_size, _) = axis.pack(&child.widget.get_size());
                let along_child_bounds = axis.along(&child.bounds.max);

                matches!(along_widget_size, SizeOp::Fill { .. })
                    && axis.along(&child.size) < along_child_bounds
            })
            .collect();

        // This distributes the remaining size to all the Fill children.
        while along_content > 0.0 && !fill_children.is_empty() {
            let portion = along_content / fill_children.len() as f32;
            let mut consumed = 0.0;

            fill_children.retain_mut(|child| {
                let along_margin = axis.along(&child.widget.get_margin()) as f32;
                let size = axis.along_mut(&mut child.size);
                let max = axis.along(&child.bounds.max);

                let remaining = max - (*size + along_margin);
                let grow = remaining.min(portion).max(0.0);

                *size += grow;
                consumed += grow;

                remaining > portion
            });

            if consumed <= f32::EPSILON {
                break;
            }

            along_content -= consumed;
        }
    // Shrink
    } else if along_content < 0.0 {
        // Along shrink children.
        let mut shrink_children: Vec<_> = node
            .children
            .iter_mut()
            .filter(|child| {
                // Skip if parent should ignore.
                if child
                    .widget
                    .get_layout()
                    .flags
                    .contains(LayoutFlags::PARENT_IGNORE)
                {
                    return false;
                }

                let (along_widget_size, _) = axis.pack(&child.widget.get_size());
                let along_child_bounds = axis.along(&child.bounds.min);

                along_widget_size.shrinkable() && axis.along(&child.size) > along_child_bounds
            })
            .collect();

        while along_content < 0.0 && !shrink_children.is_empty() {
            let portion = (along_content / shrink_children.len() as f32).abs();
            let mut consumed = 0.0;

            shrink_children.retain_mut(|child| {
                let size = axis.along_mut(&mut child.size);
                let min = axis.along(&child.bounds.min);

                let remaining = *size - min;
                let shrink = remaining.min(portion).max(0.0);

                *size -= shrink;
                consumed -= shrink;

                remaining > portion
            });

            if consumed.abs() <= f32::EPSILON {
                break;
            }

            along_content -= consumed;
        }
    }

    // Children last.
    node.children.iter_mut().for_each(resolve_fill);
}

fn wrap<T, R: Renderer + 'static>(node: &mut Node<T, R>) -> bool {
    let mut wrapped = node
        .children
        .iter_mut()
        .fold(false, |acc, child| wrap(child) | acc);

    let layout = node.widget.get_layout();

    if !layout.flags.contains(LayoutFlags::WRAP) || node.children.is_empty() {
        return wrapped;
    }

    let along_limit = layout
        .axis
        .along(&node.bounds.max)
        .min(layout.axis.along(&node.size))
        .max(0.0);

    let (along_widget_fill, across_widget_fill) = layout
        .axis
        .pack(&(HORIZONTAL_BLANK_FILL, VERTICAL_BLANK_FILL));
    let along_widget_fit_fill = layout
        .axis
        .along(&(HORIZONTAL_BLANK_FIT_FILL, VERTICAL_BLANK_FIT_FILL));

    // Always fill.
    // This phantom node represents the container where the lines will reside.
    let mut lines_node = Node::from_element(
        across_widget_fill.into(),
        &mut None,
        &node.bounds,
        node.widget.get_padding(),
    );

    // Fit Fill, to maintain line height.
    let mut line = Node::from_element(
        along_widget_fit_fill.into(),
        &mut None,
        &lines_node.bounds,
        Default::default(),
    );
    let mut along_offset: f32 = 0.0;
    for child in std::mem::take(&mut node.children) {
        if child
            .widget
            .get_layout()
            .flags
            .contains(LayoutFlags::PARENT_IGNORE)
        {
            line.children.push(child);
            continue;
        }

        // The pinned children should not be wrapped or forgotten.
        if !child.widget.get_position().is_dynamic() {
            node.children.push(child);
            continue;
        }

        let along_child_size =
            layout.axis.along(&child.size) + layout.axis.along(&child.widget.get_margin()) as f32;

        // WRAP.
        if !line.children.is_empty() && along_offset + along_child_size > along_limit {
            along_offset = 0.0;

            let mut new_line = Node::from_element(
                along_widget_fit_fill.into(),
                &mut None,
                &lines_node.bounds,
                Default::default(),
            );

            std::mem::swap(&mut line, &mut new_line);
            lines_node.children.push(new_line);
        }

        along_offset += along_child_size;

        line.children.push(child);
    }

    if !line.children.is_empty() {
        lines_node.children.push(line);
    }

    let lines = lines_node.children.len();
    // No need to wrap. So we ignore the phantom nodes.
    if lines == 1 {
        node.children
            .extend(std::mem::take(&mut lines_node.children[0].children));
    } else if lines > 1 {
        // We change the last line to fill.
        lines_node.children.last_mut().unwrap().widget = along_widget_fill.into();
        node.children.push(lines_node);

        wrapped = true;
    }

    wrapped
}

fn resolve_position<T, R: Renderer + 'static>(
    node: &mut Node<T, R>,
    overlay_nodes: &mut Vec<Node<T, R>>,
    mut clip_rect: Option<Rect>,
) {
    let layout = node.widget.get_layout();
    let padding = node.widget.get_padding();

    let content_rect = Rect {
        position: Point {
            x: node.position.x + padding.left as f32,
            y: node.position.y + padding.top as f32,
        },
        size: Size {
            width: (node.size.width - padding.get_horizontal() as f32).max(0.0),
            height: (node.size.height - padding.get_vertical() as f32).max(0.0),
        },
    };

    clip_rect = match clip_rect {
        Some(parent_clip) => parent_clip.intersection(&content_rect),
        None => Some(content_rect),
    };

    let child_is_culled = |child: &Node<T, R>| {
        let child_rect = Rect {
            position: Point {
                x: child.position.x,
                y: child.position.y,
            },
            size: Size {
                width: child.size.width,
                height: child.size.height,
            },
        };

        clip_rect.is_some_and(|clip| !child_rect.intersects(&clip))
    };

    let mut along_offset = 0.0;

    let children = std::mem::take(&mut node.children);
    node.children.reserve(children.len());

    for mut child in children {
        if child.size.width <= 0.0 || child.size.height <= 0.0 {
            continue;
        }

        let overlay = layout.flags.contains(LayoutFlags::OVERLAY);

        match child.widget.get_position() {
            Position::Dynamic => {
                let margin = child.widget.get_margin();
                let along_child_pos = layout.axis.along_mut(&mut child.position);

                *along_child_pos += along_offset;

                child.position.x += content_rect.position.x + margin.left as f32;
                child.position.y += content_rect.position.y + margin.top as f32;

                let child_extent = layout.axis.along(&(
                    child.size.width + margin.get_horizontal() as f32,
                    child.size.height + margin.get_vertical() as f32,
                ));

                along_offset += child_extent;
            }
            Position::Pinned {
                position,
                parent_relative,
            } => {
                child.position.x = position.x as f32;
                child.position.y = position.y as f32;

                if parent_relative {
                    child.position.x += node.position.x;
                    child.position.y += node.position.y;
                }
            }
        }

        if overlay {
            resolve_position(&mut child, overlay_nodes, None);
            overlay_nodes.push(child);
        } else if !child_is_culled(&child) {
            resolve_position(&mut child, overlay_nodes, clip_rect);
            node.children.push(child);
        }
    }
}

fn resolve_scaling<T, R: Renderer>(node: &mut Node<T, R>, scale: f32) {
    node.size.width *= scale;
    node.size.height *= scale;
    node.position.x *= scale;
    node.position.y *= scale;

    node.children
        .iter_mut()
        .for_each(|c| resolve_scaling(c, scale));
}

// BlankWidget is used to make wrapping easier. With more cost.
#[derive(Clone, Copy)]
struct BlankWidget {
    layout_axis: LayoutAxis,
    size: Size<SizeOp>,
}
impl<T, R: Renderer> Widget<T, R> for BlankWidget {
    #[inline]
    fn get_position(&self) -> crate::types::Position {
        crate::types::Position::Dynamic
    }

    #[inline]
    fn get_size(&self) -> crate::types::Size<SizeOp> {
        self.size
    }

    #[inline]
    fn get_layout(&self) -> crate::types::Layout {
        crate::types::Layout {
            axis: self.layout_axis,
            flags: LayoutFlags::empty(),
        }
    }

    #[inline]
    fn render(
        &self,
        _: crate::types::Point,
        _: crate::types::Size,
    ) -> Vec<crate::renderer::RenderCommand<R>> {
        vec![]
    }
}
impl<T, R: Renderer> From<BlankWidget> for Element<T, R> {
    #[inline]
    fn from(value: BlankWidget) -> Self {
        Self::new(value)
    }
}

static HORIZONTAL_BLANK_FILL: BlankWidget = BlankWidget {
    layout_axis: LayoutAxis::Horizontal,
    size: Size {
        width: SizeOp::fill(false),
        height: SizeOp::fill(false),
    },
};

static VERTICAL_BLANK_FILL: BlankWidget = BlankWidget {
    layout_axis: LayoutAxis::Vertical,
    size: Size {
        width: SizeOp::fill(false),
        height: SizeOp::fill(false),
    },
};

static HORIZONTAL_BLANK_FIT_FILL: BlankWidget = BlankWidget {
    layout_axis: LayoutAxis::Horizontal,
    size: Size {
        width: SizeOp::fill(false),
        height: SizeOp::fit(false),
    },
};

static VERTICAL_BLANK_FIT_FILL: BlankWidget = BlankWidget {
    layout_axis: LayoutAxis::Vertical,
    size: Size {
        width: SizeOp::fit(false),
        height: SizeOp::fill(false),
    },
};
