use crate::{
    types::{LayoutAxis, Node, Point, Rect, Size, SizeOp},
    widget::{Element, Widget},
};

pub(crate) fn layout<'a, T>(root: &'a Element<'a, T>, scale: f32) -> Node<'a, T> {
    let mut node = root.into();

    resolve_fit(&mut node);
    resolve_fill(&mut node);

    // This was the stable way I found to make wrapping decent.
    if wrap(&mut node) {
        resolve_fit(&mut node);
        resolve_fill(&mut node);
    }

    resolve_position(&mut node, None);
    resolve_scaling(&mut node, scale);

    node
}

fn resolve_fit<'a, T>(node: &mut Node<'a, T>) {
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
        let child_margin = child.widget.get_margin();
        let (along_child_size, across_child_size) = axis.pack(&(
            child.size.width + child_margin.horizontal() as f32,
            child.size.height + child_margin.vertical() as f32,
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
    node.size.width += padding.horizontal() as f32;
    node.size.height += padding.vertical() as f32;

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

fn resolve_fill<'a, T>(node: &mut Node<'a, T>) {
    let axis = node.widget.get_layout().axis;

    let (along_padding, across_padding) = axis.pack(&node.widget.get_padding());
    let (along_size, across_size) = axis.pack(&node.size);

    let (mut along_content, across_content) = (
        along_size - along_padding as f32,
        across_size - across_padding as f32,
    );

    for child in &mut node.children {
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

    // Along fill children.
    let mut fill_children: Vec<_> = node
        .children
        .iter_mut()
        .filter(|child| {
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

    // Children last.
    node.children.iter_mut().for_each(resolve_fill);
}

fn wrap<'a, T>(node: &mut Node<'a, T>) -> bool {
    let mut wrapped = node.children.iter_mut().any(wrap);

    let layout = node.widget.get_layout();

    if !layout.wrap || node.children.is_empty() {
        return wrapped;
    }

    let along_limit = layout.axis.along(&node.bounds.max).max(0.0);

    let (along_widget_fill, across_widget_fill) = layout.axis.pack(&(
        &HORIZONTAL_BLANK_FILL as &dyn Widget<T>,
        &VERTICAL_BLANK_FILL as &dyn Widget<T>,
    ));
    let along_widget_fit_fill = layout.axis.along(&(
        &HORIZONTAL_BLANK_FIT_FILL as &dyn Widget<T>,
        &VERTICAL_BLANK_FIT_FILL as &dyn Widget<T>,
    ));

    // Always fill.
    // This phantom node represents the container where the lines will reside.
    let mut lines_node =
        Node::from_widget(across_widget_fill, &node.bounds, node.widget.get_padding());

    // Fit Fill, to maintain line height.
    let mut line = Node::from_widget(
        along_widget_fit_fill,
        &lines_node.bounds,
        Default::default(),
    );
    let mut along_offset: f32 = 0.0;
    for child in std::mem::take(&mut node.children) {
        let along_child_size =
            layout.axis.along(&child.size) + layout.axis.along(&child.widget.get_margin()) as f32;

        // WRAP.
        if !line.children.is_empty() && along_offset + along_child_size > along_limit {
            along_offset = 0.0;

            let mut new_line = Node::from_widget(
                along_widget_fit_fill,
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
        std::mem::swap(&mut node.children, &mut lines_node.children[0].children);
    } else if lines > 1 {
        // We change the last line to fill.
        lines_node.children.last_mut().unwrap().widget = along_widget_fill;
        node.children.push(lines_node);

        wrapped = true;
    }

    wrapped
}

fn resolve_position<'a, T>(node: &mut Node<'a, T>, mut clip_rect: Option<Rect>) {
    let layout = node.widget.get_layout();
    let padding = node.widget.get_padding();

    let content_rect = Rect {
        position: Point {
            x: node.position.x + padding.left as f32,
            y: node.position.y + padding.top as f32,
        },
        size: Size {
            width: (node.size.width - padding.horizontal() as f32).max(0.0),
            height: (node.size.height - padding.vertical() as f32).max(0.0),
        },
    };

    if !layout.overflow {
        clip_rect = match clip_rect {
            Some(parent_clip) => parent_clip.intersection(&content_rect),
            None => Some(content_rect),
        };
    }

    let mut along_offset = 0.0;
    node.children.retain_mut(|child| {
        let margin = child.widget.get_margin();

        let along_child_pos = layout.axis.along_mut(&mut child.position);

        *along_child_pos += along_offset;

        child.position.x += content_rect.position.x + margin.left as f32;
        child.position.y += content_rect.position.y + margin.top as f32;

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

        if let Some(clip) = clip_rect
            && !child_rect.intersects(&clip)
        {
            return false;
        }

        let child_extent = layout.axis.along(&(
            child.size.width + margin.horizontal() as f32,
            child.size.height + margin.vertical() as f32,
        ));

        along_offset += child_extent;

        resolve_position(child, clip_rect);

        true
    });
}

fn resolve_scaling<'a, T>(node: &mut Node<'a, T>, scale: f32) {
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
impl<T> Widget<T> for BlankWidget {
    #[inline]
    fn get_id(&self) -> crate::widget::WidgetId {
        "__INTERNAL_BLANK_WIDGET__".into()
    }

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
            overflow: true,
            wrap: false,
        }
    }

    #[inline]
    fn get_children(&self) -> &[Element<'_, T>] {
        &[]
    }

    #[inline]
    fn render(
        &self,
        _: crate::types::Point,
        _: crate::types::Size,
    ) -> Vec<crate::renderer::RenderCommand> {
        vec![]
    }

    #[inline]
    fn on_input(&self, _: ()) -> Option<crate::types::Response<T>> {
        None
    }
}

static HORIZONTAL_BLANK_FILL: BlankWidget = BlankWidget {
    layout_axis: LayoutAxis::Horizontal,
    size: Size {
        width: SizeOp::fill(),
        height: SizeOp::fill(),
    },
};

static VERTICAL_BLANK_FILL: BlankWidget = BlankWidget {
    layout_axis: LayoutAxis::Vertical,
    size: Size {
        width: SizeOp::fill(),
        height: SizeOp::fill(),
    },
};

static HORIZONTAL_BLANK_FIT_FILL: BlankWidget = BlankWidget {
    layout_axis: LayoutAxis::Horizontal,
    size: Size {
        width: SizeOp::fill(),
        height: SizeOp::fit(),
    },
};

static VERTICAL_BLANK_FIT_FILL: BlankWidget = BlankWidget {
    layout_axis: LayoutAxis::Vertical,
    size: Size {
        width: SizeOp::fit(),
        height: SizeOp::fill(),
    },
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::widget::*;

    #[derive(Default)]
    struct Div<'a, T = ()> {
        children: Vec<Element<'a, T>>,
        size: Size<SizeOp>,
        layout: Layout,
    }
    impl<'a, T> Div<'a, T> {
        fn new() -> Self {
            Self {
                size: Size::default(),
                layout: Layout::default(),
                children: vec![],
            }
        }

        fn size(mut self, width: impl Into<SizeOp>, height: impl Into<SizeOp>) -> Self {
            self.size.width = width.into();
            self.size.height = height.into();

            self
        }

        fn horizontal(mut self) -> Self {
            self.layout.axis = LayoutAxis::Horizontal;

            self
        }

        fn vertical(mut self) -> Self {
            self.layout.axis = LayoutAxis::Vertical;

            self
        }

        fn wrap(mut self) -> Self {
            self.layout.wrap = true;

            self
        }

        fn children(mut self, children: impl IntoIterator<Item = Element<'a, T>>) -> Self {
            let iter = children.into_iter();

            self.children.extend(iter);

            self
        }
    }
    impl<'a, T> Widget<T> for Div<'a, T> {
        #[inline]
        fn get_id(&self) -> WidgetId {
            "__INTERNAL_TESTING__".into()
        }

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
            self.layout
        }

        fn get_children(&self) -> &[Element<'a, T>] {
            &self.children
        }

        fn on_input(&self, _: ()) -> Option<Response<T>> {
            None
        }

        fn render(&self, _: Point, _: Size) -> Vec<crate::renderer::RenderCommand> {
            vec![]
        }
    }

    fn div<'a>() -> Div<'a, ()> {
        Div::new()
    }

    fn assert_f32_eq(left: f32, right: f32) {
        assert!(
            (left - right).abs() < 1e-6,
            "left = {left}, right = {right}"
        );
    }

    #[test]
    fn fit_horizontal_sums_child_widths_and_uses_tallest_child() {
        let root = div()
            .horizontal()
            .size(SizeOp::fit(), SizeOp::fit())
            .children([
                div().size(100, 20).into_element(),
                div().size(50, 30).into_element(),
            ])
            .into_element();

        let node = layout(&root, 1.0);

        assert_f32_eq(node.size.width, 150.0);
        assert_f32_eq(node.size.height, 30.0);
    }

    #[test]
    fn fit_vertical_sums_child_heights_and_uses_widest_child() {
        let root = div()
            .vertical()
            .size(SizeOp::fit(), SizeOp::fit())
            .children([
                div().size(100, 20).into_element(),
                div().size(50, 30).into_element(),
            ])
            .into_element();

        let node = layout(&root, 1.0);

        assert_f32_eq(node.size.width, 100.0);
        assert_f32_eq(node.size.height, 50.0);
    }

    #[test]
    fn fixed_size_widget_is_left_unchanged() {
        let root = div()
            .horizontal()
            .size(250, 80)
            .children([
                div().size(100, 20).into_element(),
                div().size(50, 30).into_element(),
            ])
            .into_element();

        let node = layout(&root, 1.0);

        assert_f32_eq(node.size.width, 250.0);
        assert_f32_eq(node.size.height, 80.0);
    }

    #[test]
    fn horizontal_fill_children_share_remaining_width() {
        let root = div()
            .horizontal()
            .size(300, 100)
            .children([
                div().size(100, 20).into_element(),
                div().size(SizeOp::fill(), 20).into_element(),
                div().size(SizeOp::fill(), 20).into_element(),
            ])
            .into_element();

        let node = layout(&root, 1.0);

        assert_f32_eq(node.children[0].size.width, 100.0);
        assert_f32_eq(node.children[1].size.width, 100.0);
        assert_f32_eq(node.children[2].size.width, 100.0);
    }

    #[test]
    fn vertical_fill_children_share_remaining_height() {
        let root = div()
            .vertical()
            .size(100, 300)
            .children([
                div().size(20, 100).into_element(),
                div().size(20, SizeOp::fill()).into_element(),
                div().size(20, SizeOp::fill()).into_element(),
            ])
            .into_element();

        let node = layout(&root, 1.0);

        assert_f32_eq(node.children[0].size.height, 100.0);
        assert_f32_eq(node.children[1].size.height, 100.0);
        assert_f32_eq(node.children[2].size.height, 100.0);
    }

    #[test]
    fn cross_axis_fill_expands_to_parent_size() {
        let root = div()
            .horizontal()
            .size(300, 80)
            .children([div().size(50, SizeOp::fill()).into_element()])
            .into_element();

        let node = layout(&root, 1.0);

        println!("{:#?}", node.children[0].size);

        assert_f32_eq(node.children[0].size.width, 50.0);
        assert_f32_eq(node.children[0].size.height, 80.0);
    }

    #[test]
    fn horizontal_positions_accumulate_widths() {
        let root = div()
            .horizontal()
            .size(300, 100)
            .children([
                div().size(50, 20).into_element(),
                div().size(70, 20).into_element(),
                div().size(30, 20).into_element(),
            ])
            .into_element();

        let node = layout(&root, 1.0);

        assert_f32_eq(node.children[0].position.x, 0.0);
        assert_f32_eq(node.children[1].position.x, 50.0);
        assert_f32_eq(node.children[2].position.x, 120.0);
    }

    #[test]
    fn vertical_positions_accumulate_heights() {
        let root = div()
            .vertical()
            .size(100, 300)
            .children([
                div().size(20, 20).into_element(),
                div().size(20, 30).into_element(),
                div().size(20, 40).into_element(),
            ])
            .into_element();

        let node = layout(&root, 1.0);

        assert_f32_eq(node.children[0].position.y, 0.0);
        assert_f32_eq(node.children[1].position.y, 20.0);
        assert_f32_eq(node.children[2].position.y, 50.0);
    }

    #[test]
    fn nested_layout_accumulates_positions() {
        let root = div()
            .horizontal()
            .size(300, 100)
            .children([
                div()
                    .horizontal()
                    .size(SizeOp::fit(), SizeOp::fit())
                    .children([
                        div().size(40, 20).into_element(),
                        div().size(60, 20).into_element(),
                    ])
                    .into_element(),
                div().size(50, 20).into_element(),
            ])
            .into_element();

        let node = layout(&root, 1.0);

        assert_f32_eq(node.children[1].position.x, 100.0);
        assert_f32_eq(node.children[0].children[0].position.x, 0.0);
        assert_f32_eq(node.children[0].children[1].position.x, 40.0);
    }

    #[test]
    fn wrap_is_noop_when_everything_fits() {
        let root = div()
            .horizontal()
            .wrap()
            .size(200, 50)
            .children([
                div().size(50, 20).into_element(),
                div().size(50, 20).into_element(),
                div().size(50, 20).into_element(),
            ])
            .into_element();

        let node = layout(&root, 1.0);

        assert_eq!(node.children.len(), 3);
    }

    #[test]
    fn wrap_creates_multiple_lines_when_needed() {
        let root = div()
            .horizontal()
            .wrap()
            .size(100, 50)
            .children([
                div().size(60, 20).into_element(),
                div().size(60, 20).into_element(),
                div().size(30, 20).into_element(),
            ])
            .into_element();

        let node = layout(&root, 1.0);

        assert_eq!(node.children.len(), 1);

        let lines_node = &node.children[0];

        assert_eq!(lines_node.children.len(), 2);
        assert_eq!(lines_node.children[0].children.len(), 1);
        assert_eq!(lines_node.children[1].children.len(), 2);
    }

    #[test]
    fn fit_parent_accounts_for_fill_children_after_fill_pass() {
        let root = div()
            .horizontal()
            .size(SizeOp::fit(), SizeOp::fit())
            .children([
                div().size(100, 20).into_element(),
                div().size(SizeOp::fill(), 20).into_element(),
            ])
            .into_element();

        let node = layout(&root, 1.0);

        assert!(node.size.width >= 100.0);
    }
}
