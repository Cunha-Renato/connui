use crate::{prelude::*, renderer::Renderer};

pub(crate) fn layout<T, R: Renderer>(node: &mut Node<T, R>) {
    resolve_initial(node, &Default::default(), Default::default());
    resolve_fit(node);
    resolve_fill(node);

    // This was the stable way I found to make wrapping decent.
    if wrap(node) {
        resolve_initial(node, &Default::default(), Default::default());
        resolve_fit(node);
        resolve_fill(node);
    }

    let mut overlay = Vec::new();
    resolve_position(node, &mut overlay);
    node.children.extend(overlay);
}

fn resolve_initial<T, R: Renderer>(
    node: &mut Node<T, R>,
    bounds: &Bounds,
    padding: Sides<LogicalPixel>,
) {
    node.position = Point::default();
    node.size = node.widget.get_size().as_logical().map(|lp| lp.as_float());

    let margin = node.widget.get_margin();
    let child_padding = node.widget.get_padding();

    let mut bounds = bounds
        .width(
            node.widget.get_size().width,
            padding.get_horizontal(),
            margin.get_horizontal(),
        )
        .height(
            node.widget.get_size().height,
            padding.get_vertical(),
            margin.get_vertical(),
        );

    for child in &mut node.children {
        resolve_initial(child, &bounds, child_padding);
    }

    // Don't let this element's min size shrink below the largest child.
    // TODO: Maybe not calc this if not shrink.
    // FIXME: This has a bug!
    for child in &node.children {
        if child.widget.get_position().is_dynamic() {
            let child_margin = child.widget.get_margin();
            let child_min_w = child.bounds.min.width + child_margin.get_horizontal();
            let child_min_h = child.bounds.min.height + child_margin.get_vertical();

            bounds.min.width = bounds.min.width.max(child_min_w).min(bounds.max.width);
            bounds.min.height = bounds.min.height.max(child_min_h).min(bounds.max.height);
        }
    }

    node.bounds = bounds;
}

fn resolve_fit<T, R: Renderer>(node: &mut Node<T, R>) {
    // Children first.
    node.children.iter_mut().for_each(resolve_fit);

    let axis = node.widget.get_layout().axis;
    let (along_widget_size, across_widget_size) = axis.pack(&node.widget.get_size());

    // Absolute size, no need for this pass.
    if !along_widget_size.is_dynamic() && !across_widget_size.is_dynamic() {
        return;
    }

    // Initialize the size.
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
            child.size.width + child_margin.get_horizontal().as_float(),
            child.size.height + child_margin.get_vertical().as_float(),
        ));

        if along_widget_size.is_dynamic() {
            *along_size += along_child_size;
        }
        if across_widget_size.is_dynamic() {
            *across_size = (*across_size).max(across_child_size);
        }
    }

    // Padding.
    let padding = node.widget.get_padding();
    node.size.width += padding.get_horizontal().as_float();
    node.size.height += padding.get_vertical().as_float();

    // Clamp to bounds.
    node.size.width = node.size.width.clamp(
        node.bounds.min.width.as_float(),
        node.bounds.max.width.as_float(),
    );
    node.size.height = node.size.height.clamp(
        node.bounds.min.height.as_float(),
        node.bounds.max.height.as_float(),
    );
}

fn resolve_fill<T, R: Renderer>(node: &mut Node<T, R>) {
    let axis = node.widget.get_layout().axis;

    let (along_padding, across_padding) =
        axis.pack(&node.widget.get_padding().map(|lp| lp.as_float()));
    let (along_size, across_size) = axis.pack(&node.size);

    let (mut along_content, across_content) =
        (along_size - along_padding, across_size - across_padding);

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

        let along_child_size = axis.along(&child.size);
        let (along_margin, across_margin) =
            axis.pack(&child.widget.get_margin().map(|lp| lp.as_float()));
        along_content -= along_child_size + along_margin;

        // Across fill.
        if let SizeOp::Fill { max, .. } = axis.across(&child.widget.get_size()) {
            *axis.across_mut(&mut child.size) =
                (across_content - across_margin).min(max.as_float());
        };
    }

    // RESIZING
    // Grow
    if along_content > LogicalPixel::new(0.0) {
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

                let along_widget_size = axis.along(&child.widget.get_size());
                let along_child_size = axis.along(&child.size);
                let along_child_bounds = axis.along(&child.bounds.max).as_float();

                matches!(along_widget_size, SizeOp::Fill { max, .. } if along_child_size < along_child_bounds && along_child_size < max.as_float())
            })
            .collect();

        // This distributes the remaining size to all the Fill children.
        while along_content > LogicalPixel::new(0.0) && !fill_children.is_empty() {
            let portion = along_content / LogicalPixel::new(fill_children.len() as f32);
            let mut consumed = LogicalPixel::new(0.0);

            fill_children.retain_mut(|child| {
                let along_margin = axis.along(&child.widget.get_margin()).as_float();
                let size = axis.along_mut(&mut child.size);
                let max = axis.along(&child.bounds.max).as_float();

                let remaining = max - *size + along_margin;
                let grow = remaining.min(portion);

                *size += grow;
                consumed += grow;

                remaining > portion
            });

            if consumed <= LogicalPixel::new(f32::EPSILON) {
                break;
            }

            along_content -= consumed;
        }
    // Shrink
    } else if along_content < LogicalPixel::new(0.0) {
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

                let along_widget_size = axis.along(&child.widget.get_size());
                let along_child_bounds = axis.along(&child.bounds.min).as_float();

                along_widget_size.shrinkable() && axis.along(&child.size) > along_child_bounds
            })
            .collect();

        while along_content < LogicalPixel::new(0.0) && !shrink_children.is_empty() {
            let portion = (along_content / LogicalPixel::new(shrink_children.len() as f32)).abs();
            let mut consumed = LogicalPixel::new(0.0);

            shrink_children.retain_mut(|child| {
                let size = axis.along_mut(&mut child.size);
                let min = axis.along(&child.bounds.min).as_float();

                let remaining = *size - min;
                let shrink = remaining.min(portion).max(LogicalPixel::new(0.0));

                *size -= shrink;
                consumed -= shrink;

                remaining > portion
            });

            if consumed.abs() <= LogicalPixel::new(f32::EPSILON) {
                break;
            }

            along_content -= consumed;
        }
    }

    // Children last.
    node.children.iter_mut().for_each(resolve_fill);
}

fn wrap<T, R: Renderer>(node: &mut Node<T, R>) -> bool {
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
        .as_float()
        .min(layout.axis.along(&node.size))
        .max(LogicalPixel::new(0.0));

    let (along_widget_fill, across_widget_fill) = layout
        .axis
        .pack(&(HORIZONTAL_BLANK_FILL, VERTICAL_BLANK_FILL));
    let along_widget_fit_fill = layout
        .axis
        .along(&(HORIZONTAL_BLANK_FIT_FILL, VERTICAL_BLANK_FIT_FILL));

    // Always fill.
    // This phantom node represents the container where the lines will reside.
    let mut lines_node = Node::from_element(across_widget_fill.into(), &mut None);

    // Fit Fill, to maintain line height.
    let mut line = Node::from_element(along_widget_fit_fill.into(), &mut None);
    let mut along_offset = LogicalPixel::new(0.0);

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

        let along_child_size = layout.axis.along(&child.size)
            + layout.axis.along(&child.widget.get_margin()).as_float();

        // WRAP.
        if !line.children.is_empty() && along_offset + along_child_size > along_limit {
            along_offset = LogicalPixel::new(0.0);

            let mut new_line = Node::from_element(along_widget_fit_fill.into(), &mut None);

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

fn resolve_position<T, R: Renderer>(node: &mut Node<T, R>, overlay_nodes: &mut Vec<Node<T, R>>) {
    let layout = node.widget.get_layout();
    let content_rect = compute_content_rect(node);
    let overlay = layout.flags.contains(LayoutFlags::OVERLAY);

    let mut along_offset = LogicalPixel::new(0.0);

    let children = std::mem::take(&mut node.children);
    node.children.reserve(children.len());

    for mut child in children {
        match child.widget.get_position() {
            Position::Dynamic => {
                let margin = child.widget.get_margin();
                let along_child_pos = layout.axis.along_mut(&mut child.position);

                *along_child_pos += along_offset;

                child.position.x += content_rect.position.x + margin.left.as_float();
                child.position.y += content_rect.position.y + margin.top.as_float();

                let child_extent = layout.axis.along(&(
                    child.size.width + margin.get_horizontal().as_float(),
                    child.size.height + margin.get_vertical().as_float(),
                ));

                along_offset += child_extent;
            }
            Position::Pinned {
                position,
                parent_relative,
            } => {
                child.position.x = position.x.as_float();
                child.position.y = position.y.as_float();

                if parent_relative {
                    child.position.x += node.position.x;
                    child.position.y += node.position.y;
                }
            }
        }

        resolve_position(&mut child, overlay_nodes);

        if overlay {
            overlay_nodes.push(child);
        } else {
            node.children.push(child);
        }
    }
}

pub(crate) fn compute_content_rect<T, R: Renderer>(
    node: &Node<T, R>,
) -> Rect<LogicalPixel<f32>, LogicalPixel<f32>> {
    let padding = node.widget.get_padding();

    Rect {
        position: Point {
            x: node.position.x + padding.left.as_float(),
            y: node.position.y + padding.top.as_float(),
        },
        size: Size {
            width: (node.size.width - padding.get_horizontal().as_float())
                .max(LogicalPixel::new(0.0)),
            height: (node.size.height - padding.get_vertical().as_float())
                .max(LogicalPixel::new(0.0)),
        },
    }
}
// BlankWidget is used to make wrapping easier. With more cost.
#[derive(Clone, Copy)]
struct BlankWidget {
    layout_axis: LayoutAxis,
    size: Size<SizeOp>,
}
impl<T, R: Renderer> Widget<T, R> for BlankWidget {
    #[inline]
    fn get_position(&self) -> Position {
        Position::Dynamic
    }

    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.size
    }

    #[inline]
    fn get_layout(&self) -> Layout {
        Layout {
            axis: self.layout_axis,
            flags: LayoutFlags::empty(),
        }
    }

    #[inline]
    fn render(
        &mut self,
        _: Rect,
        _: Rect<u32, u32>,
        renderer: &mut R,
        children: &mut [Node<T, R>],
    ) {
        for child in children {
            child.render(renderer);
        }
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
