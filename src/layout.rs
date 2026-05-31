use crate::{
    types::{LayoutAxis, Node, Size, SizeOp},
    widget::{Element, Widget},
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

    node.size = node.widget.get_size().as_f32();
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

    // node.size.width = node
    //     .size
    //     .width
    //     .clamp(node.bounds.min.width, node.bounds.max.width);
    // node.size.height = node
    //     .size
    //     .height
    //     .clamp(node.bounds.min.height, node.bounds.max.height);
}

fn resolve_fill<'a, T>(node: &mut Node<'a, T>) {
    let widget_layout = node.widget.get_layout();
    let (along_size, across_size) = widget_layout.axis.pack(node.size.width, node.size.height);

    let mut along_avail_size = along_size;
    let mut along_fill_children = vec![];

    for child in &mut node.children {
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
            along_fill_children.push(child);
        }
    }

    let along_fill_children_len = along_fill_children.len();
    if along_fill_children_len > 0 && along_avail_size > 0.0 {
        let portion = along_avail_size / along_fill_children_len as f32;

        for along_fill_child in along_fill_children {
            let child_size = widget_layout.axis.along(
                &mut along_fill_child.size.width,
                &mut along_fill_child.size.height,
            );

            *child_size += portion;
        }
    }

    node.children.iter_mut().for_each(|c| resolve_fill(c));
}

fn wrap<'a, T>(node: &mut Node<'a, T>) {
    for child in &mut node.children {
        wrap(child);
    }

    let widget_layout = node.widget.get_layout();

    if !widget_layout.wrap || node.children.is_empty() {
        return;
    }

    let along_bounds = widget_layout
        .axis
        .along(node.bounds.max.width, node.bounds.max.height);

    let mut along_offset: f32 = 0.0;
    let mut across_offset: f32 = 0.0;

    let (along_widget, across_widget) = widget_layout.axis.pack(
        &HORIZONTAL_BLANK as &dyn Widget<T>,
        &VERTICAL_BLANK as &dyn Widget<T>,
    );

    let mut wrap_node = Node::<'a, T>::from(across_widget);
    wrap_node.bounds = node.bounds;

    let mut line = Node::from(along_widget);
    for child in std::mem::take(&mut node.children) {
        let (along_child_size, across_child_size) =
            widget_layout.axis.pack(child.size.width, child.size.height);

        // WRAP.
        if along_offset + along_child_size > along_bounds {
            along_offset = 0.0;
            across_offset = 0.0;

            let mut new_line = Node::from(along_widget);

            std::mem::swap(&mut line, &mut new_line);
            wrap_node.children.push(new_line);
        }

        along_offset += along_child_size;
        across_offset = across_offset.max(across_child_size);

        line.children.push(child);
    }

    if !line.children.is_empty() {
        wrap_node.children.push(line);
    }

    let wrap_node_children_len = wrap_node.children.len();
    if wrap_node_children_len == 1 {
        std::mem::swap(&mut node.children, &mut wrap_node.children[0].children);
    } else if wrap_node_children_len > 1 {
        node.children.push(wrap_node);
        resolve_fit(node);
        resolve_fill(node);
    }
}

fn resolve_position<'a, T>(node: &mut Node<'a, T>) {
    let widget_layout = node.widget.get_layout();
    let mut along_offset = 0.0;

    for child in &mut node.children {
        // if !matches!(child.widget.get_position(), Position::Dynamic) {
        //     continue;
        // }

        let along_child_size = widget_layout
            .axis
            .along(child.size.width, child.size.height);
        let along_child_position = widget_layout
            .axis
            .along(&mut child.position.x, &mut child.position.y);

        *along_child_position += along_offset;
        along_offset += along_child_size;

        child.position.x += node.position.x;
        child.position.y += node.position.y;

        resolve_position(child);
    }
}

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
            overflow: false,
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

static HORIZONTAL_BLANK: BlankWidget = BlankWidget {
    layout_axis: LayoutAxis::Horizontal,
    size: Size {
        width: SizeOp::Fill,
        height: SizeOp::Fill,
    },
};

static VERTICAL_BLANK: BlankWidget = BlankWidget {
    layout_axis: LayoutAxis::Vertical,
    size: Size {
        width: SizeOp::Fill,
        height: SizeOp::Fill,
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
            .size(SizeOp::Fit, SizeOp::Fit)
            .children([
                div().size(100, 20).into_element(),
                div().size(50, 30).into_element(),
            ])
            .into_element();

        let node = layout(&root);

        assert_f32_eq(node.size.width, 150.0);
        assert_f32_eq(node.size.height, 30.0);
    }

    #[test]
    fn fit_vertical_sums_child_heights_and_uses_widest_child() {
        let root = div()
            .vertical()
            .size(SizeOp::Fit, SizeOp::Fit)
            .children([
                div().size(100, 20).into_element(),
                div().size(50, 30).into_element(),
            ])
            .into_element();

        let node = layout(&root);

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

        let node = layout(&root);

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
                div().size(SizeOp::Fill, 20).into_element(),
                div().size(SizeOp::Fill, 20).into_element(),
            ])
            .into_element();

        let node = layout(&root);

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
                div().size(20, SizeOp::Fill).into_element(),
                div().size(20, SizeOp::Fill).into_element(),
            ])
            .into_element();

        let node = layout(&root);

        assert_f32_eq(node.children[0].size.height, 100.0);
        assert_f32_eq(node.children[1].size.height, 100.0);
        assert_f32_eq(node.children[2].size.height, 100.0);
    }

    #[test]
    fn cross_axis_fill_expands_to_parent_size() {
        let root = div()
            .horizontal()
            .size(300, 80)
            .children([div().size(50, SizeOp::Fill).into_element()])
            .into_element();

        let node = layout(&root);

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

        let node = layout(&root);

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

        let node = layout(&root);

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
                    .size(SizeOp::Fit, SizeOp::Fit)
                    .children([
                        div().size(40, 20).into_element(),
                        div().size(60, 20).into_element(),
                    ])
                    .into_element(),
                div().size(50, 20).into_element(),
            ])
            .into_element();

        let node = layout(&root);

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

        let node = layout(&root);

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

        let node = layout(&root);

        assert_eq!(node.children.len(), 1);

        let wrap_node = &node.children[0];

        assert_eq!(wrap_node.children.len(), 2);
        assert_eq!(wrap_node.children[0].children.len(), 1);
        assert_eq!(wrap_node.children[1].children.len(), 2);
    }

    #[test]
    fn fit_parent_accounts_for_fill_children_after_fill_pass() {
        let root = div()
            .horizontal()
            .size(SizeOp::Fit, SizeOp::Fit)
            .children([
                div().size(100, 20).into_element(),
                div().size(SizeOp::Fill, 20).into_element(),
            ])
            .into_element();

        let node = layout(&root);

        assert!(node.size.width >= 100.0);
    }
}
