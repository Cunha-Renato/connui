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
